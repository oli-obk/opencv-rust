extern crate pkg_config;
extern crate glob;
extern crate gcc;

use glob::glob;
use std::process::Command;
use std::path::{ PathBuf };
use std::fs;
use std::fs::{ File, read_dir };
use std::ffi::OsString;
use std::io::Write;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let opencv_env = std::env::var("OPENCV").unwrap();

    let (actual_opencv, include_paths) = if cfg!(windows) {
        (
            PathBuf::from(opencv_env.clone() + "/build/include/opencv2"),
            vec![
                PathBuf::from(opencv_env.clone() + "/build/include")
            ],
        )
    } else {
        let opencv = pkg_config::Config::new().find("opencv").unwrap();
        let mut search_paths = opencv.include_paths.clone();
        search_paths.push(PathBuf::from("/usr/include"));
        let search_opencv = search_paths.iter().map( |p| {
            let mut path = PathBuf::from(p);
            path.push("opencv2");
            path
        }).find( { |path| read_dir(path).is_ok() });
        (
            search_opencv.expect("Could not find opencv2 dir in pkg-config includes"),
            opencv.include_paths,
        )
    };

    println!("OpenCV lives in {:?}", actual_opencv);
    println!("Generating code in {}", out_dir);

    let mut gcc = gcc::Config::new();
    gcc.flag("-std=c++0x");
    for path in &include_paths {
        gcc.include(path);
    }

    for entry in glob(&(out_dir.clone() + "/*")).unwrap() {
        fs::remove_file(entry.unwrap()).unwrap()
    }

    let modules = vec![
        ("core", vec!["core/types_c.h", "core/core.hpp" ]), // utility, base
        ("imgproc", vec![ "imgproc/types_c.h", "imgproc/imgproc_c.h",
                            "imgproc/imgproc.hpp" ]),
        ("highgui", vec![   "highgui/cap_ios.h",
                            "highgui/highgui.hpp",
                            "highgui/highgui_c.h",
                            //"highgui/ios.h"
                        ]),
        ("features2d", vec![ "features2d/features2d.hpp" ]),
        ("photo", vec!["photo/photo_c.h", "photo/photo.hpp" ]),
        ("video", vec![ "video/tracking.hpp", "video/video.hpp",
                        "video/background_segm.hpp"]),
        ("objdetect", vec![ "objdetect/objdetect.hpp" ]),
        ("calib3d", vec![ "calib3d/calib3d.hpp"]),
    ];

    let mut types = PathBuf::from(&out_dir);
    types.push("common_opencv.h");
    {
        let mut types = File::create(types).unwrap();
        for ref m in modules.iter() {
            write!(&mut types, "#include <opencv2/{}/{}.hpp>\n", m.0, m.0).unwrap();
        }
    }

    let mut types = PathBuf::from(&out_dir);
    types.push("types.h");
    {
        let mut types = File::create(types).unwrap();
        write!(&mut types, "#include <cstddef>\n").unwrap();
    }

    for ref module in modules.iter() {
        let mut cpp = PathBuf::from(&out_dir);
        cpp.push(module.0);
        cpp.set_extension("cpp");

        if !Command::new("python")
                            .args(&["gen_rust.py", "hdr_parser.py", &*out_dir, module.0])
                            .args(&(module.1.iter().map(|p| {
                                let mut path = actual_opencv.clone();
                                path.push(p);
                                path.into_os_string()
                            }).collect::<Vec<OsString>>()[..]))
                           .status().unwrap().success() {
            panic!();
        }

        gcc.file(cpp);
    }

    let mut return_types = PathBuf::from(&out_dir);
    return_types.push("return_types.h");
    let mut hub_return_types = File::create(return_types).unwrap();
    for entry in glob(&(out_dir.clone() + "/cv_return_value_*.type.h")).unwrap() {
        writeln!(&mut hub_return_types, r#"#include "{}""#,
            entry.unwrap().file_name().unwrap().to_str().unwrap()).unwrap();
    }
    drop(hub_return_types); // make sure the file is saved and closed before gcc looks for it

    for entry in glob("native/*.cpp").unwrap() {
        gcc.file(entry.unwrap());
    }
    for entry in glob(&(out_dir.clone() + "/*.type.cpp")).unwrap() {
        gcc.file(entry.unwrap());
    }

    gcc.cpp(true).include(".").include(&out_dir);
    if cfg!(windows) {
        gcc.flag("/EHsc")
    } else {
        gcc.flag("-Wno-c++11-extensions")
    };
    for inc in &include_paths {
        gcc.include(inc);
    }

    gcc.compile("libocvrs.a");

    for ref module in &modules {
        if cfg!(windows) {
            println!("running cl");
            let mut gcc = gcc::Config::new();
            gcc.flag("/EHsc");
            gcc.cpp(true);
            gcc.include(std::env::current_dir().unwrap());
            for inc in &include_paths {
                let mut path = std::env::current_dir().unwrap();
                path.push(inc);
                gcc.include(path);
            }
            let mut cmd = gcc.get_compiler().to_command();
            cmd.arg(format!("{}/{}.consts.cpp", out_dir, module.0));
            cmd.current_dir(&out_dir);
            println!("{:?}", cmd);
            let out = cmd.output().unwrap();
            println!("{}", String::from_utf8(out.stdout).unwrap());
            println!("{}", String::from_utf8(out.stderr).unwrap());
            assert!(cmd.status().unwrap().success());
            let mut e = Command::new(format!("{}/{}.consts", out_dir, module.0));
            e.current_dir(&out_dir);
            println!("{:?}", e);
            let out = e.output().unwrap();
            assert!(out.status.success());
            let mut file = File::create(format!("{}/{}.consts.rs", out_dir, module.0)).unwrap();
            file.write(&out.stdout).unwrap();
        } else {
            let e = Command::new("sh").current_dir(&out_dir).arg("-c").arg(
                format!("g++ {}.consts.cpp -o {}.consts `pkg-config --cflags --libs opencv`",
                    module.0, module.0)
            ).status().unwrap();
            assert!(e.success());
            let e = Command::new("sh").current_dir(&out_dir).arg("-c").arg(
                format!("./{}.consts > {}.consts.rs", module.0, module.0)
            ).status().unwrap();
            assert!(e.success());
        }
    }

    let mut hub_filename = PathBuf::from(&out_dir);
    hub_filename.push("hub.rs");
    {
        let mut hub = File::create(hub_filename).unwrap();
        for ref module in &modules {
            writeln!(&mut hub, r#"pub mod {};"#, module.0).unwrap();
        }
        writeln!(&mut hub, r#"pub mod types {{"#).unwrap();
        writeln!(&mut hub, "  use libc::{{ c_void, c_char, size_t }};").unwrap();
        for entry in glob(&(out_dir.clone() + "/*.type.rs")).unwrap() {
            writeln!(&mut hub, r#"  include!(concat!(env!("OUT_DIR"), "/{}"));"#,
                entry.unwrap().file_name().unwrap().to_str().unwrap()).unwrap();
        }
        writeln!(&mut hub, r#"}}"#).unwrap();
        writeln!(&mut hub, "#[doc(hidden)] pub mod sys {{").unwrap();
        writeln!(&mut hub, "  use libc::{{ c_void, c_char, size_t }};").unwrap();
        for entry in glob(&(out_dir.clone() + "/*.rv.rs")).unwrap() {
            writeln!(&mut hub, r#"  include!(concat!(env!("OUT_DIR"), "/{}"));"#,
                entry.unwrap().file_name().unwrap().to_str().unwrap()).unwrap();
        }
        for ref module in &modules {
            writeln!(&mut hub, r#"  include!(concat!(env!("OUT_DIR"), "/{}.externs.rs"));"#, module.0).unwrap();
        }
        writeln!(&mut hub, "}}\n").unwrap();
    }
    println!("cargo:rustc-link-lib=ocvrs");
    if cfg!(windows) {
        for lib in glob(&(opencv_env.clone() + "/build/x64/vc12/lib/opencv_*2411.lib")).unwrap() {
            let mut lib = lib.unwrap();
            lib.set_extension("");
            println!("cargo:rustc-link-lib={}", lib.to_str().unwrap());
        }
    }
}
