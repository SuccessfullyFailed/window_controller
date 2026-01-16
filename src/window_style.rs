use crate::WindowController;



pub struct WindowStyle {
	window:WindowController,
	style_flags:u32,
	extended_style_flags:u32,
	always_on_top:bool,
	target_position:Option<[i32; 4]>
}
impl WindowStyle {

	/* CONSTRUCTOR METHODS */

	/// Create a new style.
	pub fn new(window:WindowController) -> WindowStyle {
		use winapi::um::winuser::{ GetWindowLongPtrW, GWL_STYLE, GWL_EXSTYLE };
		use winapi::shared::windef::HWND__;

		let hwnd:*mut HWND__ = window.hwnd();
		unsafe { 
			WindowStyle {
				window,
				style_flags: GetWindowLongPtrW(hwnd, GWL_STYLE) as u32,
				extended_style_flags: GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32,
				always_on_top: false,
				target_position: None
			}
		}
	}



	/* GENERAL USAGE METHODS */

	/// Set the given styling flags.
	pub fn set_style(&mut self, flags:u32, extended_flags:u32) {
		self.style_flags |= flags;
		self.extended_style_flags |= extended_flags;
	}

	/// Remove the given styling flags.
	pub fn remove_style(&mut self, flags:u32, extended_flags:u32) {
		self.style_flags &= !flags;
		self.extended_style_flags &= !extended_flags;
	}

	/// Apply the updated changes.
	pub fn apply(&self) {
		use winapi::um::winuser::{ SetWindowPos, SetWindowLongPtrW, SWP_NOMOVE, SWP_NOSIZE, SWP_FRAMECHANGED, SWP_NOACTIVATE, GWL_STYLE, GWL_EXSTYLE, HWND_TOPMOST, HWND_NOTOPMOST };

		unsafe {
			SetWindowLongPtrW(self.window.hwnd(), GWL_STYLE, self.style_flags as isize);
			SetWindowLongPtrW(self.window.hwnd(), GWL_EXSTYLE, self.extended_style_flags as isize);

			let (position, u_flags) = match self.target_position {
				Some(target_position) => (target_position, 0),
				None => ([0; 4], SWP_NOMOVE | SWP_NOSIZE | SWP_FRAMECHANGED | SWP_NOACTIVATE)
			};
			SetWindowPos(self.window.hwnd(), if self.always_on_top { HWND_TOPMOST } else { HWND_NOTOPMOST }, position[0], position[1], position[0] + position[2], position[1] + position[3], u_flags);
		}
	}



	/* SPECIFIC USAGE METHODS */

	/// Set a trans-color to the window.
	pub fn set_transcolor(&mut self, color:u32) -> &mut Self {
		use winapi::um::winuser::{ GetDC, GetWindowLongPtrW, SetLayeredWindowAttributes, ReleaseDC, GWL_EXSTYLE, WS_EX_LAYERED, WS_EX_TRANSPARENT, LWA_COLORKEY };
		use winapi::shared::windef::HDC__;

		unsafe {
			let color_bbggrr:u32 = (color & 0xFF000000) | ((color & 0xFF) << 16) | (((color >> 8) & 0xFF) << 8) | ((color >> 16) & 0xFF);

			self.set_style(0, WS_EX_LAYERED | WS_EX_TRANSPARENT);
			
			let device_context:*mut HDC__ = GetDC(self.window.hwnd());
			GetWindowLongPtrW(self.window.hwnd(), GWL_EXSTYLE);
			let _success:i32 = SetLayeredWindowAttributes(self.window.hwnd(), color_bbggrr, 0, LWA_COLORKEY);
			ReleaseDC(self.window.hwnd(), device_context)
		};

		self
	}

	/// Toggle the caption of the window.
	pub fn set_caption(&mut self, show_caption:bool) -> &mut Self {
		use winapi::um::winuser::WS_CAPTION;

		if show_caption {
			self.set_style(WS_CAPTION, 0);
		} else {
			self.remove_style(WS_CAPTION, 0);
		}

		self
	}

	/// Toggle the caption of the window.
	pub fn set_always_on_top(&mut self, always_on_top:bool) -> &mut Self {
		self.always_on_top = always_on_top;
		self
	}

	/// Set the [x, y, w, h] position of the window.
	pub fn set_position(&mut self, position:[i32; 4]) -> &mut Self {
		self.target_position = Some(position);
		self
	}
}
impl Drop for WindowStyle {
	fn drop(&mut self) {
		self.apply();
	}
}