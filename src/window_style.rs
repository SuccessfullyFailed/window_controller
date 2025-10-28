use crate::WindowController;



pub struct WindowStyle {
	window:WindowController,
	style_flags:u32,
	extended_style_flags:u32
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
				extended_style_flags: GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32
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
		use winapi::um::winuser::{ SetWindowPos, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SWP_FRAMECHANGED, SWP_NOACTIVATE };

		unsafe {
			SetWindowPos(self.window.hwnd(), std::ptr::null_mut(), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED | SWP_NOACTIVATE);
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
		use winapi::um::winuser::WS_EX_TOPMOST;
		
		if always_on_top {
			self.set_style(0, WS_EX_TOPMOST);
		} else {
			self.remove_style(0, WS_EX_TOPMOST);
		}

		self
	}
}
impl Drop for WindowStyle {
	fn drop(&mut self) {
		self.apply();
	}
}