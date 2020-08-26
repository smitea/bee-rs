  
use std::ffi::OsString;

const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(windows)]
fn main() -> windows_service::Result<()> {
    use windows_service::{
        service::{ServiceAccess, ServiceErrorControl, ServiceInfo, ServiceStartType, ServiceType},
        service_manager::{ServiceManager, ServiceManagerAccess},
    };

    let args:Vec<OsString> = std::env::args().map(|val| OsString::from(val)).collect();

    let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service_binary_path = ::std::env::current_exe()
        .unwrap()
        .with_file_name(format!("{}.exe",PKG_NAME));

    let service_info = ServiceInfo {
        name: OsString::from(PKG_NAME),
        display_name: OsString::from(format!("{} {}", PKG_NAME,PKG_VERSION)),
        service_type: ServiceType::OWN_PROCESS,
        start_type: ServiceStartType::AutoStart,
        error_control: ServiceErrorControl::Normal,
        executable_path: service_binary_path,
        launch_arguments: args.clone(),
        dependencies: vec![],
        account_name: None,
        account_password: None,
    };
    let _ = service_manager.create_service(&service_info, ServiceAccess::empty())?;
    Ok(())
}

#[cfg(not(windows))]
fn main() {
    panic!("This program is only intended to run on Windows.");
}