//! Where every thread is: `VICTORY_STACKS=<seconds>[,<seconds>...]` prints
//! each thread's unwound stack as `module+rva` that many seconds after
//! [`dump_stacks_on_schedule`], to find what a title waits on.

use std::ffi::c_void;

use windows_sys::Wdk::System::Threading::{
    NtQueryInformationThread, ThreadBasicInformation, ThreadQuerySetWin32StartAddress,
};
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE, LocalFree};
use windows_sys::Win32::System::Diagnostics::Debug::{
    CONTEXT, CONTEXT_FULL_AMD64, GetThreadContext,
};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, TH32CS_SNAPTHREAD, THREADENTRY32, Thread32First, Thread32Next,
};
use windows_sys::Win32::System::Threading::{
    GetCurrentProcessId, GetCurrentThreadId, GetThreadDescription, OpenThread, ResumeThread,
    SuspendThread, THREAD_GET_CONTEXT, THREAD_QUERY_INFORMATION, THREAD_SUSPEND_RESUME,
};

use crate::crash::{MAX_FRAMES, locate, unwind_into};

/// Reads `VICTORY_STACKS` and, if set, starts a thread that prints every
/// thread's stack at each listed time (seconds from now).
pub fn dump_stacks_on_schedule() {
    let Ok(list) = std::env::var("VICTORY_STACKS") else {
        return;
    };
    let times = schedule(&list);
    let start = std::time::Instant::now();
    std::thread::spawn(move || {
        for at in times {
            std::thread::sleep(at.saturating_sub(start.elapsed()));
            dump_stacks();
        }
    });
}

/// The times in a `VICTORY_STACKS` list (seconds, comma-separated), in
/// order. Negative times are now; times no `Duration` holds (`inf`, `NaN`,
/// `1e20`) are left out instead of panicking the dump thread.
fn schedule(list: &str) -> Vec<std::time::Duration> {
    let mut times: Vec<_> = list
        .split(',')
        .filter_map(|t| t.trim().parse::<f64>().ok())
        .filter(|t| !t.is_nan())
        .filter_map(|t| std::time::Duration::try_from_secs_f64(t.max(0.0)).ok())
        .collect();
    times.sort();
    times
}

/// `THREAD_BASIC_INFORMATION` (ntddk).
#[repr(C)]
struct BasicInfo {
    exit_status: i32,
    teb: *const usize,
    client_id: [usize; 2],
    affinity: usize,
    priority: i32,
    base_priority: i32,
}

/// `GetThreadContext` wants a 16-byte aligned context.
#[repr(C, align(16))]
struct Aligned(CONTEXT);

/// Prints every other thread's stack to stderr. Each thread is suspended
/// only while its context is read and unwound into a buffer allocated
/// beforehand (it may hold the heap lock).
pub fn dump_stacks() {
    // SAFETY: plain queries and a snapshot of this process's threads.
    let (me, process, snapshot) = unsafe {
        (
            GetCurrentThreadId(),
            GetCurrentProcessId(),
            CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0),
        )
    };
    if snapshot == INVALID_HANDLE_VALUE {
        return;
    }
    let mut ids = Vec::new();
    // SAFETY: zeroed is a valid THREADENTRY32; its size is set before use.
    let mut entry: THREADENTRY32 = unsafe { std::mem::zeroed() };
    entry.dwSize = size_of::<THREADENTRY32>() as u32;
    // SAFETY: `snapshot` is open; `entry` is sized.
    let mut more = unsafe { Thread32First(snapshot, &mut entry) } != 0;
    while more {
        if entry.th32OwnerProcessID == process && entry.th32ThreadID != me {
            ids.push(entry.th32ThreadID);
        }
        // SAFETY: as above.
        more = unsafe { Thread32Next(snapshot, &mut entry) } != 0;
    }
    // SAFETY: we opened it.
    unsafe { CloseHandle(snapshot) };
    eprintln!("VIctory: stacks of {} threads", ids.len());
    for id in ids {
        // SAFETY: opens a thread of our own process.
        let thread = unsafe {
            OpenThread(
                THREAD_SUSPEND_RESUME | THREAD_GET_CONTEXT | THREAD_QUERY_INFORMATION,
                0,
                id,
            )
        };
        if thread.is_null() {
            continue;
        }
        let (frames, (priority, base)) = frames_of(thread);
        eprintln!(
            "  thread {id} \"{}\" start {} priority {priority} (base {base})",
            name_of(thread),
            locate(start_of(thread))
        );
        for (i, pc) in frames.into_iter().enumerate() {
            eprintln!("    #{i:<2} {}", locate(pc as usize));
        }
        // SAFETY: we opened it.
        unsafe { CloseHandle(thread) };
    }
}

/// The thread's frames and its current and base scheduling priority (0-31).
fn frames_of(thread: HANDLE) -> (Vec<u64>, (i32, i32)) {
    let mut frames = Vec::with_capacity(MAX_FRAMES);
    let mut info: BasicInfo = BasicInfo {
        exit_status: 0,
        teb: std::ptr::null(),
        client_id: [0; 2],
        affinity: 0,
        priority: 0,
        base_priority: 0,
    };
    // Suspended first: a running thread may exit and free its TEB between
    // the query and the read below.
    // SAFETY: a thread of this process that is not this one.
    if unsafe { SuspendThread(thread) } == u32::MAX {
        return (frames, (0, 0));
    }
    // SAFETY: writes `info`, whose layout and size are the system's.
    let status = unsafe {
        NtQueryInformationThread(
            thread,
            ThreadBasicInformation,
            (&raw mut info).cast(),
            size_of::<BasicInfo>() as u32,
            std::ptr::null_mut(),
        )
    };
    if status != 0 || info.teb.is_null() {
        // SAFETY: suspended above.
        unsafe { ResumeThread(thread) };
        return (frames, (0, 0));
    }
    // SAFETY: the TEB of a suspended thread of this process: its NT_TIB
    // holds the stack base at +8 and the stack limit at +0x10.
    let (high, low) = unsafe { (info.teb.add(1).read(), info.teb.add(2).read()) };
    // SAFETY: zeroed is a valid CONTEXT.
    let mut context = Aligned(unsafe { std::mem::zeroed() });
    context.0.ContextFlags = CONTEXT_FULL_AMD64;
    // SAFETY: reads and resumes the thread suspended above; nothing until
    // the resume allocates or takes a lock the thread could hold besides
    // the unwinder's own.
    unsafe {
        if GetThreadContext(thread, &mut context.0) != 0 {
            unwind_into(context.0, low, high, &mut frames);
        }
        ResumeThread(thread);
    }
    (frames, (info.priority, info.base_priority))
}

fn start_of(thread: HANDLE) -> usize {
    let mut start = 0usize;
    // SAFETY: writes one pointer-sized start address.
    unsafe {
        NtQueryInformationThread(
            thread,
            ThreadQuerySetWin32StartAddress,
            (&raw mut start).cast(),
            size_of::<usize>() as u32,
            std::ptr::null_mut(),
        )
    };
    start
}

fn name_of(thread: HANDLE) -> String {
    let mut name = std::ptr::null_mut();
    // SAFETY: the system allocates the string; we free it with LocalFree.
    unsafe {
        if GetThreadDescription(thread, &mut name) < 0 || name.is_null() {
            return String::new();
        }
        let len = (0..).take_while(|&i| *name.add(i) != 0).count();
        let text = String::from_utf16_lossy(std::slice::from_raw_parts(name, len));
        LocalFree(name.cast::<c_void>());
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn another_threads_stack_unwinds_to_its_start() {
        let (tx, rx) = std::sync::mpsc::channel();
        let (stop_tx, stop_rx) = std::sync::mpsc::channel::<()>();
        let parked = std::thread::spawn(move || {
            // SAFETY: plain query.
            tx.send(unsafe { GetCurrentThreadId() }).unwrap();
            stop_rx.recv().ok();
        });
        let id = rx.recv().unwrap();
        // SAFETY: a thread of this process.
        let thread = unsafe {
            OpenThread(
                THREAD_SUSPEND_RESUME | THREAD_GET_CONTEXT | THREAD_QUERY_INFORMATION,
                0,
                id,
            )
        };
        assert!(!thread.is_null());
        let (frames, _) = frames_of(thread);
        // Waiting in the kernel, several frames down from the thread start.
        assert!(frames.len() >= 3, "{frames:?}");
        assert!(start_of(thread) != 0);
        // SAFETY: opened above.
        unsafe { CloseHandle(thread) };
        dump_stacks();
        stop_tx.send(()).unwrap();
        parked.join().unwrap();
    }

    /// Times no `Duration` holds are left out instead of panicking the dump
    /// thread (`from_secs_f64` panicked on `inf`); the rest come in order.
    #[test]
    fn schedules_skip_times_out_of_range() {
        let secs = |list| {
            schedule(list)
                .iter()
                .map(|d| d.as_secs_f64())
                .collect::<Vec<_>>()
        };
        assert_eq!(secs("30, 5,x,-2,inf,NaN,1e20"), [0.0, 5.0, 30.0]);
        assert!(schedule("").is_empty());
    }
}

/// Stack scans on demand: with `VICTORY_STACKS_FILE=<path>`, creating that
/// file prints every thread's stack scan (see [`scan_stacks`]) and deletes
/// it. Unlike the scheduled dumps it runs only when asked (after a hang),
/// and it never calls the unwinder, which faulted under Wine.
pub fn scan_stacks_on_demand() {
    let Some(path) = std::env::var_os("VICTORY_STACKS_FILE") else {
        return;
    };
    let path = std::path::PathBuf::from(path);
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_millis(500));
            if path.exists() {
                let _ = std::fs::remove_file(&path);
                scan_stacks();
            }
        }
    });
}

/// Words of a thread's stack copied while it is suspended.
const SCAN_WORDS: usize = 4096;

/// Prints, for every other thread, its instruction pointer and the words of
/// its stack (from its stack pointer, up to 32 KiB) that point into
/// executable image memory: candidate return addresses, innermost first.
/// The thread is suspended only while its context and stack are copied
/// into buffers allocated beforehand; nothing then allocates, takes a lock
/// or calls the unwinder. Addresses are named afterwards, with every
/// thread running again; image bases are printed too, in case naming
/// blocks on the loader lock.
pub fn scan_stacks() {
    use windows_sys::Win32::System::Memory::{
        MEM_IMAGE, MEMORY_BASIC_INFORMATION, PAGE_EXECUTE, PAGE_EXECUTE_READ,
        PAGE_EXECUTE_READWRITE, PAGE_EXECUTE_WRITECOPY, VirtualQuery,
    };
    // SAFETY: plain queries and a snapshot of this process's threads.
    let (me, process, snapshot) = unsafe {
        (
            GetCurrentThreadId(),
            GetCurrentProcessId(),
            CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0),
        )
    };
    if snapshot == INVALID_HANDLE_VALUE {
        return;
    }
    let mut ids = Vec::new();
    // SAFETY: zeroed is a valid THREADENTRY32; its size is set before use.
    let mut entry: THREADENTRY32 = unsafe { std::mem::zeroed() };
    entry.dwSize = size_of::<THREADENTRY32>() as u32;
    // SAFETY: `snapshot` is open; `entry` is sized.
    let mut more = unsafe { Thread32First(snapshot, &mut entry) } != 0;
    while more {
        if entry.th32OwnerProcessID == process && entry.th32ThreadID != me {
            ids.push(entry.th32ThreadID);
        }
        // SAFETY: as above.
        more = unsafe { Thread32Next(snapshot, &mut entry) } != 0;
    }
    // SAFETY: we opened it.
    unsafe { CloseHandle(snapshot) };
    // An executable image page: the base of its image, or None.
    let code_base = |address: u64| -> Option<usize> {
        // SAFETY: zeroed is a valid MEMORY_BASIC_INFORMATION.
        let mut info: MEMORY_BASIC_INFORMATION = unsafe { std::mem::zeroed() };
        // SAFETY: a query of our own address space.
        let n = unsafe {
            VirtualQuery(address as *const c_void, &mut info, size_of::<MEMORY_BASIC_INFORMATION>())
        };
        let exec = PAGE_EXECUTE | PAGE_EXECUTE_READ | PAGE_EXECUTE_READWRITE | PAGE_EXECUTE_WRITECOPY;
        (n != 0 && info.Type == MEM_IMAGE && info.Protect & exec != 0)
            .then_some(info.AllocationBase as usize)
    };
    let mut words = vec![0u64; SCAN_WORDS];
    let mut bases = std::collections::BTreeSet::new();
    let mut found: Vec<(u32, u64, Vec<u64>)> = Vec::with_capacity(ids.len());
    eprintln!("VIctory: stack scans of {} threads", ids.len());
    for id in ids {
        // SAFETY: opens a thread of our own process.
        let thread = unsafe {
            OpenThread(
                THREAD_SUSPEND_RESUME | THREAD_GET_CONTEXT | THREAD_QUERY_INFORMATION,
                0,
                id,
            )
        };
        if thread.is_null() {
            continue;
        }
        let (rip, n) = copy_stack(thread, &mut words);
        // SAFETY: we opened it.
        unsafe { CloseHandle(thread) };
        let hits: Vec<u64> = words[..n]
            .iter()
            .copied()
            .filter(|&w| code_base(w).inspect(|&b| { bases.insert(b); }).is_some())
            .collect();
        eprintln!("  thread {id:#x} rip {rip:#x} ({} return candidates)", hits.len());
        for w in &hits {
            eprintln!("    {w:#x}");
        }
        found.push((id, rip, hits));
    }
    eprintln!("VIctory: image bases {:x?}", bases);
    // Names last: GetModuleHandleExW may wait for the loader lock.
    for (id, rip, hits) in found {
        eprintln!("  thread {id:#x} at {}", locate(rip as usize));
        for w in hits.iter().take(48) {
            eprintln!("    {}", locate(*w as usize));
        }
    }
    eprintln!("VIctory: stack scans done");
}

/// The thread's instruction pointer, and how many words of its stack (from
/// its stack pointer) were copied into `words`.
fn copy_stack(thread: HANDLE, words: &mut [u64]) -> (u64, usize) {
    let mut info: BasicInfo = BasicInfo {
        exit_status: 0,
        teb: std::ptr::null(),
        client_id: [0; 2],
        affinity: 0,
        priority: 0,
        base_priority: 0,
    };
    // SAFETY: a thread of this process that is not this one.
    if unsafe { SuspendThread(thread) } == u32::MAX {
        return (0, 0);
    }
    // SAFETY: writes `info`, whose layout and size are the system's.
    let status = unsafe {
        NtQueryInformationThread(
            thread,
            ThreadBasicInformation,
            (&raw mut info).cast(),
            size_of::<BasicInfo>() as u32,
            std::ptr::null_mut(),
        )
    };
    let mut copied = (0, 0);
    if status == 0 && !info.teb.is_null() {
        // SAFETY: the TEB of a suspended thread: NT_TIB stack base at +8,
        // limit at +0x10.
        let (high, low) = unsafe { (info.teb.add(1).read(), info.teb.add(2).read()) };
        // SAFETY: zeroed is a valid CONTEXT.
        let mut context = Aligned(unsafe { std::mem::zeroed() });
        context.0.ContextFlags = CONTEXT_FULL_AMD64;
        // SAFETY: reads the suspended thread's registers into our buffer.
        if unsafe { GetThreadContext(thread, &mut context.0) } != 0 {
            let rsp = context.0.Rsp as usize & !7;
            if rsp >= low && rsp < high {
                let n = ((high - rsp) / 8).min(words.len());
                // SAFETY: [rsp, rsp + 8n) lies inside the suspended
                // thread's committed stack, below its base.
                unsafe { std::ptr::copy_nonoverlapping(rsp as *const u64, words.as_mut_ptr(), n) };
                copied = (context.0.Rip, n);
            } else {
                copied = (context.0.Rip, 0);
            }
        }
    }
    // SAFETY: suspended above.
    unsafe { ResumeThread(thread) };
    copied
}
