use core::ffi::CStr;
use core::mem::MaybeUninit;
use dds_bridge_sys as dds;

fn main() {
    let info = unsafe {
        let mut info: MaybeUninit<dds::DDSInfo> = MaybeUninit::uninit();
        dds::SetMaxThreads(0);
        dds::GetDDSInfo(info.as_mut_ptr());
        info.assume_init()
    };

    println!("DDS Version: {}.{}.{}", info.major, info.minor, info.patch);
    println!("Number of threads: {}", info.noOfThreads);

    println!(
        "Thread sizes: {}",
        unsafe { CStr::from_ptr(info.threadSizes.as_ptr()) }.to_string_lossy()
    );
}
