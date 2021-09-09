fn main() {
    windows::build! {
        Windows::Win32::UI::Shell::*,
        Windows::Win32::UI::WindowsAndMessaging::*,
        Windows::Win32::System::Registry::*,
        Windows::Win32::System::Threading::*,
    };
}