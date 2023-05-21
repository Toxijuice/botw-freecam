use crate::globals::*;
use nalgebra_glm as glm;
use windows_sys::Win32::UI::{
    Input::{
        KeyboardAndMouse::*,
        XboxController::{XInputGetState, XINPUT_STATE},
    },
    WindowsAndMessaging::MessageBoxA,
};
use colored::*;

const DEADZONE: i16 = 10000;
const MINIMUM_ENGINE_SPEED: f32 = 1e-3;

pub const INSTRUCTIONS: &str =
"Read instructions.txt for controls.";

const CARGO_VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");
const GIT_VERSION: Option<&'static str> = option_env!("GIT_VERSION");

/// Generate current version of the executable from the
/// latest git version and the cargo verison.
pub fn get_version() -> String {
    let cargo = CARGO_VERSION.unwrap_or("Unknown");
    let git = GIT_VERSION.unwrap_or("");

    return format!("{} {}", cargo, git);
}

/// Keys that aren't contained in the VirtualKeys from the Windows API.
#[repr(i32)]
#[rustfmt::skip]
#[allow(dead_code)]
pub enum Keys {
    A = 0x41, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
}

pub fn check_key_press(key: u16) -> bool {
    (unsafe { GetAsyncKeyState(key as _) } as u32) & 0x8000 != 0
}

pub fn calc_eucl_distance(a: &glm::Vec3, b: &glm::Vec3) -> f32 {
    let diff = a - b;
    glm::l2_norm(&diff)
}

#[derive(Default, Debug)]
pub struct Input {
    pub engine_speed: f32,
    // Deltas with X and Y
    pub delta_pos: (f32, f32),
    pub delta_focus: (f32, f32),

    pub delta_rotation: f32,

    pub delta_altitude: f32,

    pub change_active: bool,
    pub is_active: bool,

    pub fov: f32,

    pub deattach: bool,

    pub speed_multiplier: f32,

    pub dolly_duration: f32,
    pub dolly_increment: f32,

    pub unlock_character: bool,

    pub any_shoulder_held: bool,
    pub both_shoulders_held: bool,
    pub any_dpad_held: bool,
    pub left_thumb_held: bool,

    pub fov_preset_index: i32,
    pub rot_speed: f32,
    pub delta_sign_x: f32,
    pub delta_sign_y: f32,
    pub rot_shift_x: f32,
    pub rot_shift_y: f32,
}

impl Input {
    pub fn new() -> Input {
        Self {
            fov: 0.8726647,
            engine_speed: MINIMUM_ENGINE_SPEED,
            speed_multiplier: 1.,
            dolly_duration: 10.,
            dolly_increment: 0.01,
            fov_preset_index: -1,
            rot_speed: 0.75,
            delta_sign_x: 1.,
            delta_sign_y: 1.,
            any_shoulder_held: false,
            both_shoulders_held: false,
            any_dpad_held: false,
            left_thumb_held: false,
            rot_shift_x: 0.78539816,
            rot_shift_y: 0.6981317,
            ..Input::default()
        }
    }

    pub fn reset(&mut self) {
        self.delta_pos = (0., 0.);
        self.delta_focus = (0., 0.);
        self.delta_altitude = 0.;
        self.change_active = false;

        #[cfg(debug_assertions)]
        {
            self.deattach = false;
        }
    }

    pub fn sanitize(&mut self) {
        if self.fov < 1e-3 {
            self.fov = 0.01;
        }
        if self.fov > 3.12 {
            self.fov = 3.12;
        }

        if self.dolly_duration < 0.1 {
            self.dolly_duration = 0.1;
        }

        if self.engine_speed < MINIMUM_ENGINE_SPEED {
            self.engine_speed = MINIMUM_ENGINE_SPEED;
        }

        if self.speed_multiplier > 10. {
            self.speed_multiplier = 10.
        }

        if self.speed_multiplier < 0.01 {
            self.speed_multiplier = 0.01;
        }
    }
}

pub fn handle_keyboard(input: &mut Input) {
    macro_rules! handle_state {
            ([ $key_pos:expr, $key_neg:expr, $var:ident, $val:expr ]; $($tt:tt)*) => {
                handle_state!([$key_pos, $key_neg, $var = $val, $var = - $val]; $($tt)*);
            };

            ([ $key_pos:expr, $key_neg:expr, $pos_do:expr, $neg_do:expr ]; $($tt:tt)*) => {
                if (GetAsyncKeyState($key_pos as i32) as u32 & 0x8000) != 0 {
                    $pos_do;
                }

                if (GetAsyncKeyState($key_neg as i32) as u32 & 0x8000) != 0 {
                    $neg_do;
                }
                handle_state!($($tt)*);
            };

            () => {}
        }

    unsafe {
        handle_state! {
                // Others
                [VK_F2, VK_F3, input.change_active = true, input.change_active = false];
        }
    }

    if !input.is_active {
        return;
    }

    unsafe {
        handle_state! {
            // Position of the camer
            [Keys::W, Keys::S, input.delta_pos.1 = 0.02, input.delta_pos.1 = -0.02];
            [Keys::A, Keys::D, input.delta_pos.0 = 0.02, input.delta_pos.0 = -0.02];
            [VK_UP, VK_DOWN, input.delta_focus.1 = -0.02, input.delta_focus.1 = 0.02];
            [VK_LEFT, VK_RIGHT, input.delta_focus.0 = -0.02, input.delta_focus.0 = 0.02];

            [Keys::Q, Keys::E, input.delta_altitude -= 0.02, input.delta_altitude += 0.02];

            // Rotation
            [VK_NEXT, VK_PRIOR, input.delta_rotation += 0.02, input.delta_rotation -= 0.02];

            //  FoV
            [VK_F5, VK_F6, input.fov -= 0.02, input.fov += 0.02];

            [VK_F3, VK_F4, input.speed_multiplier -= 0.01, input.speed_multiplier += 0.01];

        }
    }

    if check_key_press(Keys::P as _) {
        input.dolly_duration += input.dolly_increment;
        input.dolly_increment *= 1.01;
        println!("Duration: {}", input.dolly_duration);
    } else if check_key_press(Keys::O as _) {
        input.dolly_duration -= input.dolly_increment;
        input.dolly_increment *= 1.01;
        println!("Duration: {}", input.dolly_duration);
    } else {
        input.dolly_increment = 0.01
    }

    if check_key_press(VK_LSHIFT) {
        input.delta_pos.0 *= 8.;
        input.delta_pos.1 *= 8.;
        input.delta_altitude *= 8.;
    }

    if check_key_press(VK_TAB) {
        input.delta_pos.0 *= 0.2;
        input.delta_pos.1 *= 0.2;
        input.delta_altitude *= 0.2;
    }

    input.delta_pos.0 *= input.speed_multiplier;
    input.delta_pos.1 *= input.speed_multiplier;
    input.delta_altitude *= input.speed_multiplier;
}

pub fn error_message(message: &str) {
    let title = String::from("Error while patching\0");
    let message = format!("{}\0", message);

    unsafe {
        MessageBoxA(0, message.as_ptr(), title.as_ptr(), 0x10);
    }
}

pub fn handle_controller(input: &mut Input, func: fn(u32, &mut XINPUT_STATE) -> u32) {
    let mut xs: XINPUT_STATE = unsafe { std::mem::zeroed() };
    func(0, &mut xs);

    let gp = xs.Gamepad;
    
    // [wButtons]
    // XINPUT_GAMEPAD_DPAD_UP 	        0x0001
    // XINPUT_GAMEPAD_DPAD_DOWN 	    0x0002
    // XINPUT_GAMEPAD_DPAD_LEFT 	    0x0004
    // XINPUT_GAMEPAD_DPAD_RIGHT 	    0x0008
    // XINPUT_GAMEPAD_START 	        0x0010
    // XINPUT_GAMEPAD_BACK 	            0x0020
    // XINPUT_GAMEPAD_LEFT_THUMB 	    0x0040
    // XINPUT_GAMEPAD_RIGHT_THUMB 	    0x0080
    // XINPUT_GAMEPAD_LEFT_SHOULDER 	0x0100
    // XINPUT_GAMEPAD_RIGHT_SHOULDER 	0x0200
    // XINPUT_GAMEPAD_A 	            0x1000
    // XINPUT_GAMEPAD_B 	            0x2000
    // XINPUT_GAMEPAD_X 	            0x4000
    // XINPUT_GAMEPAD_Y 	            0x8000

    // check camera activation
    if gp.bLeftTrigger > 150 && ((gp.wButtons & 0x2000) == 0x2000) {
        input.change_active = true;
    }

    // Update the camera changes only if it's listening
    if !input.is_active { return; }

    // modify speed
    // A
    if (gp.wButtons & 0x1000) != 0 {
        input.speed_multiplier -= 0.01;
        println!("{} {}", "Speed:".bright_white(), input.speed_multiplier.to_string().bright_blue());
    }
    // X
    if (gp.wButtons & 0x4000) != 0 {
        input.speed_multiplier += 0.01;
        println!("{} {}", "Speed:".bright_white(), input.speed_multiplier.to_string().bright_blue());
    }

    // Right shoulder
    if (gp.wButtons & (0x0200)) != 0 {
        if !input.any_shoulder_held {
            input.delta_rotation += 1.57079633; // Roll camera 90deg
            println!("{} {}", "Camera rolled".bright_white(), "90°".bright_blue());
        }
        
        input.any_shoulder_held = true;
    }
    // Left shoulder
    else if (gp.wButtons & (0x0100)) != 0 {
        if !input.any_shoulder_held {
            input.delta_rotation += -1.57079633; // Roll camera -90deg
            println!("{} {}", "Camera rolled".bright_white(), "-90°".bright_blue());
        }

        input.any_shoulder_held = true;
    }
    else {
        input.any_shoulder_held = false;
    }

    // Both shoulders
    if (gp.wButtons & (0x0200 | 0x0100)) == (0x0200 | 0x0100) {
        if !input.both_shoulders_held {
            input.delta_rotation = 0.;
            println!("{} {}", "Camera roll".bright_white(), "reset".bright_blue());
        }

        input.both_shoulders_held = true;
    }
    else {
        input.both_shoulders_held = false;
    }

    // B
    if (gp.wButtons & 0x2000) != 0 {
        input.fov += 0.005;
        println!("{}{}", "FOV: ".bright_white(), input.fov.to_string().bright_blue());
    }

    // Y
    if (gp.wButtons & 0x8000) != 0 {
        input.fov -= 0.005;
        println!("{}{}", "FOV: ".bright_white(), input.fov.to_string().bright_blue());
    }

    // Left thumb
    if (gp.wButtons & (0x0040)) != 0 {
        if !input.left_thumb_held {
            input.fov_preset_index += 1;
            input.fov_preset_index %= 4; // Preset count

            // This is dumb but I don't know how arrays work in Rust
            if input.fov_preset_index == 0 {
                input.fov = 1.02477892093;
                println!("{}{}", "FOV Preset: ".bright_white(), "90°".bright_blue());
            } else if input.fov_preset_index == 1 {
                input.fov = 0.8726647;
                println!("{}{}", "FOV Preset: ".bright_white(), "~79° (default)".bright_blue());
            } else if input.fov_preset_index == 2 {
                input.fov = 0.814931329159;
                println!("{}{}", "FOV Preset: ".bright_white(), "75°".bright_blue());
            } else if input.fov_preset_index == 3 {
                input.fov = 0.457822332671;
                println!("{}{}", "FOV Preset: ".bright_white(), "45°".bright_blue());
            }
            
            println!("{}{}", "FOV: ".bright_white(), input.fov.to_string().bright_blue());
        }
        
        input.left_thumb_held = true;
    }
    else {
        input.left_thumb_held = false;
    }

    input.delta_altitude += -(gp.bLeftTrigger as f32) / 5e3;
    input.delta_altitude += (gp.bRightTrigger as f32) / 5e3;

    macro_rules! dead_zone {
        ($val:expr) => {
            if ($val < DEADZONE) && ($val > -DEADZONE) {
                0
            } else {
                $val
            }
        };
    }

    input.delta_pos.0 = -(dead_zone!(gp.sThumbLX) as f32) / ((i16::MAX as f32) * 1e2) * input.speed_multiplier;
    input.delta_pos.1 =  (dead_zone!(gp.sThumbLY) as f32) / ((i16::MAX as f32) * 1e2) * input.speed_multiplier;

    input.delta_focus.0 = (dead_zone!(gp.sThumbRX) as f32) / ((i16::MAX as f32) * 4e1);
    input.delta_focus.1 = -(dead_zone!(gp.sThumbRY) as f32) / ((i16::MAX as f32) * 4e1);

    // Negative check
    if input.delta_focus.0 < 0. { input.delta_sign_x = -1.; }
    else {  input.delta_sign_x = 1.; }

    if input.delta_focus.1 < 0. { input.delta_sign_y = -1.; }
    else {  input.delta_sign_y = 1.; }

    // Change range from 0.000 - 0.025 to 0.0 - 1.0
    input.delta_focus.0 = input.delta_focus.0 * 40.;
    input.delta_focus.1 = input.delta_focus.1 * 40.;

    // Squared curve, apply negative
    input.delta_focus.0 = input.delta_focus.0 * input.delta_focus.0 * input.delta_sign_x;
    input.delta_focus.1 = input.delta_focus.1 * input.delta_focus.1 * input.delta_sign_y;

    // Revert range to 0.000 - 0.025
    input.delta_focus.0 = input.delta_focus.0 * 0.025;
    input.delta_focus.1 = input.delta_focus.1 * 0.025;

    // Scale speed depending on fov
    input.delta_focus.0 = input.delta_focus.0 * (input.fov + 0.01) * input.rot_speed;
    input.delta_focus.1 = input.delta_focus.1 * (input.fov + 0.01) * input.rot_speed;


    // DPAD Up
    if (gp.wButtons & (0x0001)) != 0 {
        if !input.any_dpad_held {
            input.delta_focus.1 = -input.rot_shift_y;
        }
        input.any_dpad_held = true;
    } 
    // DPAD Down
    else if (gp.wButtons & (0x0002)) != 0 {
        if !input.any_dpad_held {
            input.delta_focus.1 = input.rot_shift_y;
        }
        input.any_dpad_held = true;
    }
    // DPAD Left
    else if (gp.wButtons & (0x0004)) != 0 {
        if !input.any_dpad_held {
            input.delta_focus.0 = -input.rot_shift_x;
        }
        input.any_dpad_held = true;
    } 
    // DPAD Right
    else if (gp.wButtons & (0x0008)) != 0 {
        if !input.any_dpad_held {
            input.delta_focus.0 = input.rot_shift_x;
        }
        input.any_dpad_held = true;
    }
    else {
        input.any_dpad_held = false;
    }


    if input.delta_focus.0 != 0. {
        println!("{} {}", "rot dx:".bright_white(), input.delta_focus.0.to_string().bright_blue());
    }

    if input.delta_focus.1 != 0. {
        println!("{} {}", "rot dy:".bright_white(), input.delta_focus.1.to_string().bright_blue());
    }

    input.delta_altitude *= input.speed_multiplier;
}

#[no_mangle]
pub unsafe extern "system" fn dummy_xinput(a: u32, b: &mut XINPUT_STATE) -> u32 {
    if g_camera_active != 0 {
        *b = std::mem::zeroed();
        return 0;
    }

    XInputGetState(a, b)
}
