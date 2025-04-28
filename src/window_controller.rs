use winapi::{ shared::windef::HWND, um::winuser::GetForegroundWindow };
use std::error::Error;



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

	

	/* STYLE ALTERING METHODS */

	/// Update the window.
	fn apply_style_changes(&self) {
		use winapi::um::winuser::{ SetWindowPos, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SWP_FRAMECHANGED, SWP_NOACTIVATE };

		unsafe { SetWindowPos(self.0, std::ptr::null_mut(), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED | SWP_NOACTIVATE); }
	}

	/// Update the window style.
	pub fn update_style(&self, extended:bool, style_function:&dyn Fn(u32) -> u32) {
		use winapi::um::winuser::{ GetWindowLongPtrW, SetWindowLongPtrW, GWL_STYLE, GWL_EXSTYLE };

		unsafe {
			let style_pointer:i32 = if extended { GWL_EXSTYLE } else { GWL_STYLE };
			let style:u32 = (style_function)(GetWindowLongPtrW(self.0, style_pointer) as u32);
			SetWindowLongPtrW(self.0, style_pointer, style as isize);
			self.apply_style_changes();
		}
	}

	/// Do not steal focus when activating.
	pub fn disable_focus_steal(&self) {
		use winapi::um::winuser::{ SetWindowPos, HWND_TOPMOST, SWP_NOMOVE, SWP_NOSIZE };

		unsafe { SetWindowPos(self.0, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE); }
	}

	/// Set a transcolor to the window.
	pub fn set_transcolor(&self, color:u32) {
		use winapi::um::winuser::{ GetDC, GetWindowLongPtrW, SetLayeredWindowAttributes, ReleaseDC, GWL_EXSTYLE, WS_EX_LAYERED, WS_EX_TRANSPARENT, LWA_COLORKEY };
		use winapi::shared::windef::HDC__;
		
		// Convert color from 0xAARRGGBB to 0xBBGGRR.
		let color:u32 = (color & 0xFF000000) | ((color & 0xFF) << 16) | (((color >> 8) & 0xFF) << 8) | ((color >> 16) & 0xFF);

		// Apply new style.
		unsafe {
			self.update_style(true, &|style| { style | WS_EX_LAYERED | WS_EX_TRANSPARENT });
			let device_context:*mut HDC__ = GetDC(self.0);
			GetWindowLongPtrW(self.0, GWL_EXSTYLE);
			let _success:i32 = SetLayeredWindowAttributes(self.0, color, 0, LWA_COLORKEY);
			ReleaseDC(self.0, device_context)
		};
	}

	/// Toggle the caption of the window.
	pub fn set_caption(&self, show_caption:bool) {
		use winapi::um::winuser::WS_CAPTION;

		self.update_style(false, &|style|{ if show_caption { style | WS_CAPTION } else { style & !WS_CAPTION} });
	}

	/// Toggle the caption of the window.
	pub fn set_always_on_top(&self, always_on_top:bool) {
		use winapi::um::winuser::WS_EX_TOPMOST;
		
		self.update_style(true, &|style|{ if always_on_top { style | WS_EX_TOPMOST } else { style & !WS_EX_TOPMOST} });
	}
}