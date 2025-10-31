use winapi::{ ctypes::c_void, shared::{ minwindef::DWORD, windef::{ HBITMAP__, HDC__ } }, um::{ wingdi::{ BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDIBits, SelectObject }, winuser::{ GetDC, ReleaseDC } } };
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

			// Create a device context.
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

			// Create a compatible bitmap.
			let hbitmap:*mut HBITMAP__ = CreateCompatibleBitmap(dc, bounds[2], bounds[3]);
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
			
			// BitBlt to capture the screen content.
			let result:i32 = BitBlt(hdc, -bounds[0], -bounds[1], bounds[0] + bounds[2], bounds[1] + bounds[3], dc, 0, 0, 0x00CC0020);
			if result == 0 {
				DeleteDC(hdc);
				ReleaseDC(self.hwnd(), dc);
				return Err("Image from screen result is 0.".into());
			}
			
			// Get the pixel data using GetDIBits.
			let mut bitmap_info:BITMAPINFO = mem::zeroed();
			bitmap_info.bmiHeader.biSize = mem::size_of::<BITMAPINFOHEADER>() as DWORD;
			bitmap_info.bmiHeader.biWidth = bounds[2];
			bitmap_info.bmiHeader.biHeight = -bounds[3];
			bitmap_info.bmiHeader.biPlanes = 1;
			bitmap_info.bmiHeader.biBitCount = 32;
			bitmap_info.bmiHeader.biCompression = BI_RGB;
			
			// Create a list of bits.
			let mut bits:Vec<u8> = vec![0; (bounds[2] * bounds[3] * 4) as usize];
			bits.resize((bounds[2] * bounds[3] * 4) as usize, 0u8);
			let result:i32 = GetDIBits(hdc, hbitmap, 0, bounds[3] as u32, bits.as_mut_ptr() as *mut c_void, &mut bitmap_info, DIB_RGB_COLORS);
			if result == 0 {
				DeleteDC(hdc);
				ReleaseDC(self.hwnd(), dc);
				return Err("GetDIBits failed.".into());
			}
			
			// Convert the raw pixel data to the desired format.
			let mut pixels:Vec<u32> = vec![0x00000000; (bounds[2] * bounds[3]) as usize];
			for pixel_index in 0..(bounds[2] * bounds[3]) as usize {
				pixels[pixel_index] = u32::from_be_bytes([0xFF, bits[pixel_index * 4 + 2], bits[pixel_index * 4 + 1], bits[pixel_index * 4]]);
			}
			
			// Cleanup.
			SelectObject(hdc, hold);
			DeleteObject(hbitmap as *mut _);
			DeleteDC(hdc);
			ReleaseDC(self.hwnd(), dc);


			Ok(WindowImage {
				data: pixels,
				width: bounds[2] as usize,
				height: bounds[3] as usize
			})
		}
	}
}