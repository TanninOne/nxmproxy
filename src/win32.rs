use std::{ffi::c_void, io::{Error, ErrorKind}, mem, path::Path, ptr::null_mut};

use bindings::Windows::Win32::{Foundation::{HANDLE, HINSTANCE, HWND, PWSTR}, System::Registry::{HKEY, HKEY_CLASSES_ROOT, KEY_ALL_ACCESS, KEY_QUERY_VALUE, REG_CREATE_KEY_DISPOSITION, REG_OPTION_NON_VOLATILE, REG_SZ, RRF_RT_REG_SZ, RegCloseKey, RegCreateKeyExW, RegGetValueW, RegOpenKeyExW, RegSetValueExW}, UI::Shell::{CommandLineToArgvW, SHELLEXECUTEINFOW, SHELLEXECUTEINFOW_0, ShellExecuteExW}};

// copied from
// https://github.com/microsoft/windows-samples-rs/blob/master/webview2_win32/src/pwstr.rs
fn string_from_pwstr(source: PWSTR) -> String {
    if source.is_null() {
        String::new()
    } else {
        let mut buffer = Vec::new();
        let mut pwz = source.0;

        unsafe {
            while *pwz != 0 {
                buffer.push(*pwz);
                pwz = pwz.add(1);
            }
        }

        String::from_utf16(&buffer).expect("Failed to convert from windows api")
    }
}

pub fn parse_commandline(command_line: &str) -> Result<(String, Vec<String>), Error> {
    let exe: String;
    let mut args: Vec<String> = vec![];

    unsafe {
        let mut num_args: i32 = 0;
        let parsed = CommandLineToArgvW(command_line, &mut num_args);

        exe = string_from_pwstr(*parsed);

        for i in 1..num_args {
            args.push(string_from_pwstr(*parsed.offset(i as isize)));
        }
    }

    return Ok((exe, args))
}

pub fn get_protocol_handler(protocol: &str) -> Result<String, Error> {
    let result: String;
    unsafe {
        let mut hkey: HKEY = HKEY::NULL;
        let sub_key = format!(r"{0}\shell\open\command", protocol);
        let mut res = RegOpenKeyExW(HKEY_CLASSES_ROOT, sub_key,
             0, KEY_QUERY_VALUE, & mut hkey);
        if res.0 != 0 {
            return Err(Error::from_raw_os_error(res.0));
        }

        let mut buffer: Vec<u16> = vec![0; 256];
        let mut data_size: u32 = buffer.capacity() as u32;
        // preflight to find out the required buffer size
        res = RegGetValueW(hkey, None, None, RRF_RT_REG_SZ,
              null_mut(), buffer.as_mut_ptr() as * mut c_void, & mut data_size);

        buffer.resize((data_size / 2 - 1) as usize, 0);

        // more data, buffer was too small
        if res.0 == 234 {
            res = RegGetValueW(hkey, None, None, RRF_RT_REG_SZ,
                null_mut(), buffer.as_mut_ptr() as * mut c_void, & mut data_size);
            // cut off 0 terminator
            buffer.resize((data_size / 2 - 1) as usize, 0);
        }

        if res.0 != 0 {
            return Err(Error::from_raw_os_error(res.0));
        }

        result = String::from_utf16(&buffer).expect("failed to convert registry value");

        res = RegCloseKey(hkey);

        if res.0 != 0 {
            return Err(Error::from_raw_os_error(res.0));
        }
    }

    Ok(result)
}

pub fn set_protocol_handler(protocol: &str, command: &str) -> Result<(), Error> {
    unsafe {
        let mut hkey: HKEY = HKEY::NULL;
        let sub_key = format!(r"{0}\shell\open\command", protocol);
        let mut dispo: REG_CREATE_KEY_DISPOSITION = REG_CREATE_KEY_DISPOSITION(0);
        let mut res = RegCreateKeyExW(HKEY_CLASSES_ROOT, sub_key,
            0, None, REG_OPTION_NON_VOLATILE,
            KEY_ALL_ACCESS, null_mut(), & mut hkey,
            & mut dispo);

        if res.0 != 0 {
            return Err(Error::from_raw_os_error(res.0));
        }

        let mut command_u16 = vec![];
        for ch in command.encode_utf16() {
            // little endian
            command_u16.push(ch as u8);
            command_u16.push((ch >> 8) as u8);
        }

        res = RegSetValueExW(hkey, None, 0, REG_SZ,
            command_u16.as_ptr(), command_u16.len() as u32);

        if res.0 != 0 {
            return Err(Error::from_raw_os_error(res.0));
        }

        res = RegCloseKey(hkey);

        if res.0 != 0 {
            return Err(Error::from_raw_os_error(res.0));
        }
    }

    Ok(())
}

fn to_utf16(input: &str) -> Vec<u16> {
    let mut res: Vec<u16> = input.encode_utf16().collect();
    res.push(0);
    return res;
}

pub fn spawn_elevated(exe: &str, args: Vec<&str>) -> Result<(), Error> {
    let cwd = Path::new(&exe).parent().unwrap().to_str().unwrap();

    let mut verb: Vec<u16> = to_utf16("runas");
    let mut file: Vec<u16> = to_utf16(exe);
    let mut directory: Vec<u16> = to_utf16(cwd);
    let mut parameters: Vec<u16> = to_utf16(args.join(" ").as_str());
    let mut class: Vec<u16> = vec![];

    let mut exec_info: SHELLEXECUTEINFOW = SHELLEXECUTEINFOW {
        cbSize: mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        lpVerb: PWSTR(verb.as_mut_ptr()),
        lpFile: PWSTR(file.as_mut_ptr()),
        lpDirectory: PWSTR(directory.as_mut_ptr()),
        lpParameters: PWSTR(parameters.as_mut_ptr()),
        nShow: 1,
        fMask: 0,
        hwnd: HWND(0),
        hInstApp: HINSTANCE(0),
        lpIDList: 0 as * mut c_void,
        hkeyClass: HKEY(0),
        lpClass: PWSTR(class.as_mut_ptr()),
        Anonymous: SHELLEXECUTEINFOW_0 { hIcon: HANDLE(0) },
        hProcess: HANDLE(0),
        dwHotKey: 0,
    };

    unsafe {
        if ShellExecuteExW(& mut exec_info).as_bool() {
            Ok(())
        } else {
            Err(Error::new(ErrorKind::Other, "failed to spawn"))
        }
    }
}
