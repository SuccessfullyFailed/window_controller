use winapi::{ ctypes::c_void, shared::{ minwindef::DWORD, windef::{ HBITMAP__, HDC__, POINT, RECT } }, um::{ wingdi::{ BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleBitmap, CreateCompatibleDC, DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDIBits, SelectObject }, winuser::{ ClientToScreen, GetClientRect, GetDC, GetWindowRect, PW_RENDERFULLCONTENT, PrintWindow, ReleaseDC } } };
use std::{ error::Error, mem };
use crate::WindowController;



pub struct WindowImage {
	pub data:Vec<u32>, // 0xAARRGGBB
	pub width:usize,
	pub height:usize
}
impl WindowImage {

	/// Return the data of the image in a list of rows.
	pub fn data_2d(&self) -> Vec<&[u32]> {
		self.data.chunks(self.width).collect()
	}
}



impl WindowController {

	/// Tries to create an image of the inner window. Returns a list of u32 0xAARRGGBB values.
	pub fn create_window_image(&self) -> Result<WindowImage, Box<dyn Error>> {
		let window_position:[i32; 4] = self.position();
		self.create_window_image_with_bounds([0, 0, window_position[2], window_position[3]])
	}
	
	/// Tries to create an image of a subsection of the inner window. Returns a list of u32 0xAARRGGBB values.
	pub fn create_window_image_with_bounds(&self, bounds:[i32; 4]) -> Result<WindowImage, Box<dyn Error>> {
		unsafe {

			// Validate bounds width/height.
			if bounds[2] <= 0 || bounds[3] <= 0 {
				return Err("Invalid bounds size".into());
			}

			// Determine window/client/padding.
			let mut window_bounds:RECT = mem::zeroed();
			let mut window_client_bounds:RECT = mem::zeroed();
			let mut window_topleft:POINT = POINT { x: 0, y: 0 };
			GetWindowRect(self.hwnd(), &mut window_bounds);
			GetClientRect(self.hwnd(), &mut window_client_bounds);
			ClientToScreen(self.hwnd(), &mut window_topleft);
			let padding:[i32; 4] = [
				(window_topleft.x - window_bounds.left).max(0),
				(window_topleft.y - window_bounds.top).max(0),
				(window_bounds.right - (window_topleft.x + (window_client_bounds.right - window_client_bounds.left))).max(0),
				(window_bounds.bottom - (window_topleft.y + (window_client_bounds.bottom - window_client_bounds.top))).max(0)
			];

			// Calculate padded width. Contains the requested bounds inside the client area. Bitmap must be large enough to include left and top padding.
			let padded_size:[i32; 2] = [bounds[2] + padding[0] + padding[2], bounds[3] + padding[1] + padding[3]];
			if padded_size[0] <= 0 || padded_size[1] <= 0 {
				return Err("Computed padded size is invalid".into());
			}

			// Create a device context for the window.
			let dc:*mut HDC__ = GetDC(self.hwnd());
			if dc.is_null() {
				return Err("Could not create device context".into());
			}

			// Create compatible device context.
			let hdc:*mut HDC__ = CreateCompatibleDC(dc);
			if hdc.is_null() {
				ReleaseDC(self.hwnd(), dc);
				return Err("Could not create compatible device context.".into())
			}

			// Create a compatible bitmap sized to the padded size.
			let hbitmap:*mut HBITMAP__ = CreateCompatibleBitmap(dc, padded_size[0], padded_size[1]);
			if hbitmap.is_null() {
				DeleteDC(hdc);
				ReleaseDC(self.hwnd(), dc);
				return Err("Could not create compatible bitmap.".into())
			}

			// Select the bitmap into the DC.
			let hold:*mut c_void = SelectObject(hdc, hbitmap as *mut _);
			if hold.is_null() {
				DeleteObject(hbitmap as *mut _);
				DeleteDC(hdc);
				ReleaseDC(self.hwnd(), dc);
				return Err("Could not select the bitmap in the device context.".into())
			}

			// Capture image from window to hdc. Renders the full window. Bitmap was sized to include the non-client areas.
			let result:i32 = PrintWindow(self.hwnd(), hdc, PW_RENDERFULLCONTENT);
			if result == 0 {
				SelectObject(hdc, hold);
				DeleteObject(hbitmap as *mut _);
				DeleteDC(hdc);
				ReleaseDC(self.hwnd(), dc);
				return Err("PrintWindow failed".into());
			}

			// Prepare BITMAPINFO for the padded size (top-down)
			let mut bitmap_info:BITMAPINFO = mem::zeroed();
			bitmap_info.bmiHeader.biSize = mem::size_of::<BITMAPINFOHEADER>() as DWORD;
			bitmap_info.bmiHeader.biWidth = padded_size[0];
			bitmap_info.bmiHeader.biHeight = -padded_size[1]; // Negative to get data top-down.
			bitmap_info.bmiHeader.biPlanes = 1;
			bitmap_info.bmiHeader.biBitCount = 32;
			bitmap_info.bmiHeader.biCompression = BI_RGB;

			// Allocate buffer for the full padded capture (BGRA)
			let mut bits:Vec<u8> = vec![0u8; (padded_size[0] * padded_size[1] * 4) as usize];
			let res:i32 = GetDIBits(hdc, hbitmap, 0, padded_size[1] as u32, bits.as_mut_ptr() as *mut c_void, &mut bitmap_info, DIB_RGB_COLORS);
			if res == 0 {
				SelectObject(hdc, hold);
				DeleteObject(hbitmap as *mut _);
				DeleteDC(hdc);
				ReleaseDC(self.hwnd(), dc);
				return Err("GetDIBits failed.".into());
			}


			// Collect data from image, skipping padding.
			let mut pixels:Vec<u32> = vec![0x00000000; (bounds[2] * bounds[3]) as usize];
			for output_y in 0..bounds[3] {
				for output_x in 0..bounds[2] {
					let (input_x, input_y) = (output_x + padding[0], output_y + padding[1]);
					let output_index:usize = (output_y * bounds[2] + output_x) as usize;
					let input_index:usize = (input_y * padded_size[0] + input_x) as usize;
					pixels[output_index] = u32::from_be_bytes([0xFF, bits[input_index * 4 + 2], bits[input_index * 4 + 1], bits[input_index * 4]]);
				}
			}


			// Cleanup.
			SelectObject(hdc, hold);
			DeleteObject(hbitmap as *mut _);
			DeleteDC(hdc);
			ReleaseDC(self.hwnd(), dc);

			// Return image.
			Ok(WindowImage {
				data: pixels,
				width: bounds[2] as usize,
				height: bounds[3] as usize
			})
		}
	}
}