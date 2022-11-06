use std::process::exit;

mod args_parser;
mod yaml_parser;

fn main() {
    let _path = match args_parser::parse() {
        Ok(path) => path,
        Err(e) => match e {
            args_parser::ArgsError::TooFew => {
                print_usage();
                exit(1);
            }
        },
    };

    let s = "--- !u!29 &1
OcclusionCullingSettings:
  m_ObjectHideFlags: 0
  serializedVersion: 2
  m_OcclusionBakeSettings:
    smallestOccluder: 5
    smallestHole: 0.25
    backfaceThreshold: 100
  m_SceneGUID: 00000000000000000000000000000000
  m_OcclusionCullingData: {fileID: 0}
--- !u!104 &2
RenderSettings:
  m_ObjectHideFlags: 0
  serializedVersion: 9
  m_Fog: 0
  m_FogColor: {r: 0.5, g: 0.5, b: 0.5, a: 1}
  m_FogMode: 3
  m_Component:
  - component: {fileID: 47246278}
  - component: {fileID: 47246277}
  - component: {fileID: 47246276}";
    for uo in yaml_parser::parse(s) {
        println!("{:?}", uo);
    }
}

fn print_usage() {
    eprintln!("Usage:");
}
