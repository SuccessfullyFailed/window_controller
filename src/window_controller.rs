use winapi::{ shared::windef::HWND, um::winuser::GetForegroundWindow };
use std::{ error::Error, sync::{ Mutex, MutexGuard } };
use crate::WindowStyle;



static WINDOW_COLLECTOR_LOCK:Mutex<()> = Mutex::new(());
static mut WINDOW_FILTER:Option<Box<dyn Fn(&WindowController) -> bool>> = None;
static mut STOP_AFTER_FIRST:bool = false;
static mut FOUND_WINDOWS:Vec<WindowController> = Vec::new();



#[derive(Clone, PartialEq)]
pub struct WindowController(HWND);
#[allow(dead_code)]
impl WindowController {

	/* CONSTRUCTOR METHODS */

	/// Get a controller to the current foreground window.
	pub fn active() -> WindowController {
		WindowController(unsafe { GetForegroundWindow() })
	}

	/// Get a controller from a specific hwnd.
	pub fn from_hwnd(hwnd:HWND) -> WindowController {
		WindowController(hwnd)
	}

	/// Find window by its title.
	pub fn find_by_title(title:&str) -> Option<WindowController> {
		let title:String = title.to_string();
		WindowController::find_one(move |window| window.title().contains(&title))
	}

	/// Try to find one window matching the given filter.
	pub fn find_one<T:Fn(&WindowController) -> bool + 'static>(filter:T) -> Option<WindowController> {
		let found:Vec<WindowController> = WindowController::find(filter, true);
		if found.is_empty() {
			None
		} else {
			Some(found[0].clone())
		}
	}

	/// Find all windows matching the given filter.
	pub fn find_all<T:Fn(&WindowController) -> bool + 'static>(filter:T) -> Vec<WindowController> {
		WindowController::find(filter, false)
	}

	/// Get a controller to all existing windows.
	#[allow(static_mut_refs)]
	fn find<T:Fn(&WindowController) -> bool + 'static>(filter:T, find_one:bool) -> Vec<WindowController> {
		unsafe {
			let collect_lock:MutexGuard<'_, ()> = WINDOW_COLLECTOR_LOCK.lock().unwrap();
			WINDOW_FILTER = Some(Box::new(filter));
			STOP_AFTER_FIRST = find_one;
			FOUND_WINDOWS = Vec::new();
			winapi::um::winuser::EnumWindows(Some(WindowController::externally_get_window_controllers), 0);
			let windows:Vec<WindowController> = FOUND_WINDOWS.clone();
			drop(collect_lock);
			windows
		}
	}
	#[allow(static_mut_refs)]
	unsafe extern "system" fn externally_get_window_controllers(hwnd:HWND, _control_handle:winapi::shared::minwindef::LPARAM) -> winapi::shared::minwindef::BOOL  {
		unsafe {
			let controller:WindowController = WindowController(hwnd);
			if (WINDOW_FILTER.as_ref().unwrap())(&controller) {
				FOUND_WINDOWS.push(controller);
				if STOP_AFTER_FIRST {
					return winapi::shared::minwindef::FALSE;
				}
			}
			winapi::shared::minwindef::TRUE
		}
	}



	/* ACTION METHODS */

	/// Set this window as the active one.
	pub fn activate(&self) {
		use winapi::um::winuser::{ SetForegroundWindow, SetWindowPos, HWND_TOP, SWP_SHOWWINDOW, SW_SHOW};

		if !self.is_active() {
			unsafe {
				let pos:[i32; 4] = self.position();
				SetWindowPos(self.0, HWND_TOP, pos[0], pos[1], pos[0] + pos[2], pos[1] + pos[3], SWP_SHOWWINDOW | SW_SHOW as u32);
				SetForegroundWindow(self.0);
			}
		}
	}
	
	/// Minimize the window.
	pub fn minimize(&self) {
		unsafe { winapi::um::winuser::ShowWindow(self.0, winapi::um::winuser::SW_MINIMIZE); }
	}
	
	/// Post a message to the window.
	pub fn post_message(&self, message:u32) {
		unsafe { winapi::um::winuser::PostMessageW(self.0, message, 0, 0); };
	}

	/// Move the window to a new xywh position.
	pub fn set_pos(&self, position:[i32; 4]) {
		use winapi::um::winuser::{ SetWindowPos, HWND_TOP, SWP_NOZORDER };
		unsafe { SetWindowPos(self.0, HWND_TOP, position[0], position[1], position[2], position[3], SWP_NOZORDER); }
	}
	
	/// Close the window.
	pub fn close(&self) {
		unsafe { winapi::um::winuser::PostMessageW(self.0, winapi::um::winuser::WM_CLOSE, 0, 0); }
	}



	/* PROPERTY GETTER METHODS */

	/// Get the HWND of the window.
	pub fn hwnd(&self) -> HWND {
		self.0
	}
	
	/// Check if window is active.
	pub fn is_active(&self) -> bool {
		self == &WindowController::active()
	}

	/// Check if the window exists.
	pub fn exists(&self) -> bool {
		unsafe { winapi::um::winuser::IsWindow(self.0) != 0 }
	}

	/// Check if the window is visible.
	pub fn is_visible(&self) -> bool {
		unsafe { winapi::um::winuser::IsWindowVisible(self.0) != 0 }
	}

	/// Check if the window is minimized.
	pub fn is_minimized(&self) -> bool {
		unsafe { winapi::um::winuser::IsIconic(self.0) != 0 }
	}

	/// Get the process ID of the window.
	pub fn pid(&self) -> u32 {
		let mut pid:winapi::shared::minwindef::DWORD = 0;
		unsafe { winapi::um::winuser::GetWindowThreadProcessId(self.0, &mut pid); }
		pid as u32
	}

	/// Get the ID of the window.
	pub fn id(&self) -> u32 {
		unsafe { winapi::um::winuser::GetDlgCtrlID(self.0) as u32 }
	}

	/// Get the title of the window.
	pub fn title(&self) -> String {
		unsafe {
			let mut buffer:[u16; 255] = [0u16; 255];
			let length:i32 = winapi::um::winuser::GetWindowTextW(self.0, buffer.as_mut_ptr(), buffer.len() as i32);
			(0..length as usize).map(|index| buffer[index] as u8 as char).collect::<String>()
		}
	}

	/// Get the process name of the window.
	pub fn process_name(&self) -> Result<String, Box<dyn Error>> {
		let path:String = self.exe_path()?;
		if let Some(last_node) = path.replace('\\', "/").split('/').last() {
			Ok(last_node.to_owned())
		} else {
			Err(format!("Could not get last node in path '{path}'.").into())
		}
	}

	/// Get the path of the executable the window is based on.
	pub fn exe_path(&self) -> Result<String, Box<dyn Error>> {
		use winapi::{ ctypes::c_void, um::{ handleapi::CloseHandle, processthreadsapi::OpenProcess, winbase::QueryFullProcessImageNameW, winnt::PROCESS_QUERY_LIMITED_INFORMATION } };
		use std::{ ffi::OsString, os::windows::ffi::OsStringExt };

		unsafe {
		
			// Open the process with PROCESS_QUERY_LIMITED_INFORMATION access.
			let process_handle:*mut c_void = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, self.pid());
			if process_handle.is_null() {
				return Err("Could not open process to find exe path.".into());
			}
		
			// Query the full path of the process executable.
			let mut buffer_size:u32 = 260; // MAX_PATH size
			let mut buffer:Vec<u16> = vec![0; buffer_size as usize];
			let success:i32 = QueryFullProcessImageNameW(process_handle, 0, buffer.as_mut_ptr(), &mut buffer_size);
			CloseHandle(process_handle);
			if success == 0 {
				return Err("Could not get full process image.".into());
			}
			
			// Convert the buffer into a Rust String.
			Ok(OsString::from_wide(&buffer[..buffer_size as usize]).to_string_lossy().replace('\\', "/"))
		}
	}

	/// Get the position of this window.
	pub fn position(&self) -> [i32; 4] {
		use winapi::{ shared::windef::{ RECT, POINT }, um::winuser::{ GetClientRect, ClientToScreen } };

		let mut client_rect:RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
		let mut top_left:POINT = POINT { x: 0, y: 0 };
		unsafe {
			GetClientRect(self.hwnd(), &mut client_rect);
			top_left.x = client_rect.left;
			top_left.y = client_rect.top;
			ClientToScreen(self.hwnd(), &mut top_left);
		}
		[top_left.x, top_left.y, client_rect.right - client_rect.left, client_rect.bottom - client_rect.top]
	}

	/// Do not steal focus when activating.
	pub fn disable_focus_steal(&self) {
		use winapi::um::winuser::{ SetWindowPos, HWND_TOPMOST, SWP_NOMOVE, SWP_NOSIZE };

		unsafe { SetWindowPos(self.0, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE); }
	}

	/// Get the current style.
	pub fn style(&self) -> WindowStyle {
		WindowStyle::new(self.clone())
	}
}