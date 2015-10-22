extern crate opencv;

use opencv::core;
use opencv::highgui;

fn run() -> Result<(),String> {
    let window = "video capture";
    try!(highgui::named_window(window, 1));
    let mut cam = try!(highgui::VideoCapture::device(0));
    assert!(try!(cam.is_opened()));
    loop {
        let mut frame = try!(core::Mat::new());
        assert!(try!(cam.read(&mut frame)));
        if try!(frame.size()).width > 0 {
            try!(highgui::imshow(window, &mut frame));
        } else {
            println!("zero sized frame");
        }
        if try!(highgui::wait_key(10)) > 0 {
            break;
        }
    }
    Ok(())
}

fn main() {
    run().unwrap()
}
