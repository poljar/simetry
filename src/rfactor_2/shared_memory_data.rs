pub const MAX_MAPPED_VEHICLES: usize = 128;
pub const MAX_MAPPED_IDS: usize = 512;

type String64 = [u8; 64];
type String32 = [u8; 32];
type String24 = [u8; 24];
type String18 = [u8; 18];
type String16 = [u8; 16];
type Garbage = u8;

#[repr(C, packed(4))]
#[derive(Copy, Clone, Debug, Default)]
pub struct PageVec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[repr(C, packed(4))]
#[derive(Copy, Clone, Debug)]
pub struct PageHeader {
    /// Incremented right before buffer is written to.
    pub version_update_begin: u32,
    /// Incremented after buffer write is done.
    pub version_update_end: u32,
}

#[repr(C, packed(4))]
#[derive(Copy, Clone, Debug)]
pub struct PageForceFeedback {
    pub ignored_header: PageHeader,

    /// Current FFB value reported via InternalsPlugin::ForceFeedback.
    pub force_value: f64,
}

#[repr(C, packed(4))]
#[derive(Copy, Clone, Debug)]
pub struct PageTelemetry {
    pub header: PageHeader,

    /// How many bytes of the structure were written during the last update.
    ///
    /// 0 means unknown (whole buffer should be considered as updated).
    pub bytes_updated_hint: i32,

    /// current number of vehicles
    pub num_vehicles: i32,
    pub vehicles: [PageVehicleTelemetry; MAX_MAPPED_VEHICLES],
}

#[repr(C, packed(4))]
#[derive(Copy, Clone, Debug)]
pub struct PageVehicleTelemetry {
    // Time
    /// slot ID (note that it can be re-used in multiplayer after someone leaves)    
    pub id: i32,
    /// time since last update (seconds)    
    pub delta_time: f64,
    /// game session time    
    pub elapsed_time: f64,
    /// current lap number    
    pub lap_number: i32,
    /// time this lap was started    
    pub lap_start_et: f64,
    /// current vehicle name    
    pub vehicle_name: String64,
    /// current track name    
    pub track_name: String64,

    // Position and derivatives
    /// world position in meters    
    pub pos: PageVec3,
    /// velocity (meters/sec) in local vehicle coordinates    
    pub local_vel: PageVec3,
    /// acceleration (meters/sec^2) in local vehicle coordinates    
    pub local_accel: PageVec3,

    // Orientation and derivatives
    /// rows of orientation matrix (use TelemQuat conversions if desired), also converts local    
    pub ori: [PageVec3; 3],
    // vehicle vectors into world X, Y, or Z using dot product of rows 0, 1, or 2 respectively
    /// rotation (radians/sec) in local vehicle coordinates    
    pub local_rot: PageVec3,
    /// rotational acceleration (radians/sec^2) in local vehicle coordinates    
    pub local_rot_accel: PageVec3,

    // Vehicle status
    /// -1=reverse, 0=neutral, 1+=forward gears    
    pub gear: i32,
    /// engine RPM    
    pub engine_rpm: f64,
    /// Celsius    
    pub engine_water_temp: f64,
    /// Celsius    
    pub engine_oil_temp: f64,
    /// clutch RPM    
    pub clutch_rpm: f64,

    // Driver input
    /// ranges  0.0-1.0    
    pub unfiltered_throttle: f64,
    /// ranges  0.0-1.0    
    pub unfiltered_brake: f64,
    /// ranges -1.0-1.0 (left to right)    
    pub unfiltered_steering: f64,
    /// ranges  0.0-1.0    
    pub unfiltered_clutch: f64,

    // Filtered input (various adjustments for rev or speed limiting, TC, ABS?, speed sensitive steering, clutch work for semi-automatic shifting, etc.)
    /// ranges  0.0-1.0    
    pub filtered_throttle: f64,
    /// ranges  0.0-1.0    
    pub filtered_brake: f64,
    /// ranges -1.0-1.0 (left to right)    
    pub filtered_steering: f64,
    /// ranges  0.0-1.0    
    pub filtered_clutch: f64,

    // Misc
    /// torque around steering shaft (used to be mSteeringArmForce, but that is not necessarily accurate for feedback purposes)    
    pub steering_shaft_torque: f64,
    /// deflection at front 3rd spring    
    pub front3rd_deflection: f64,
    /// deflection at rear 3rd spring    
    pub rear3rd_deflection: f64,

    // Aerodynamics
    /// front wing height    
    pub front_wing_height: f64,
    /// front ride height    
    pub front_ride_height: f64,
    /// rear ride height    
    pub rear_ride_height: f64,
    /// drag    
    pub drag: f64,
    /// front downforce    
    pub front_downforce: f64,
    /// rear downforce    
    pub rear_downforce: f64,

    // State/damage info
    /// amount of fuel (liters)    
    pub fuel: f64,
    /// rev limit    
    pub engine_max_rpm: f64,
    /// number of scheduled pitstops    
    pub scheduled_stops: u8,
    /// whether overheating icon is shown    
    pub overheating: u8,
    /// whether any parts (besides wheels) have been detached    
    pub detached: u8,
    /// whether headlights are on    
    pub headlights: u8,
    /// dent severity at 8 locations around the car (0=none, 1=some, 2=more)    
    pub dent_severity: [u8; 8],
    /// time of last impact    
    pub last_impact_et: f64,
    /// magnitude of last impact    
    pub last_impact_magnitude: f64,
    /// location of last impact    
    pub last_impact_pos: PageVec3,

    // Expanded
    /// current engine torque (including additive torque) (used to be mEngineTq, but there's little reason to abbreviate it)    
    pub engine_torque: f64,
    /// the current sector (zero-based) with the pitlane stored in the sign bit (example: entering pits from third sector gives 0x80000002)    
    pub current_sector: i32,
    /// whether speed limiter is on    
    pub speed_limiter: u8,
    /// maximum forward gears    
    pub max_gears: u8,
    /// index within brand    
    pub front_tire_compound_index: u8,
    /// index within brand    
    pub rear_tire_compound_index: u8,
    /// capacity in liters    
    pub fuel_capacity: f64,
    /// whether front flap is activated    
    pub front_flap_activated: u8,
    /// whether rear flap is activated    
    pub rear_flap_activated: u8,
    pub rear_flap_legal_status: u8,
    pub ignition_starter: u8,

    /// name of front tire compound    
    pub front_tire_compound_name: String18,
    /// name of rear tire compound    
    pub rear_tire_compound_name: String18,

    /// whether speed limiter is available    
    pub speed_limiter_available: u8,
    /// whether (hard) anti-stall is activated    
    pub anti_stall_activated: u8,
    ///
    unused: [Garbage; 2],
    /// the *visual* steering wheel range    
    pub visual_steering_wheel_range: f32,

    /// fraction of brakes on rear    
    pub rear_brake_bias: f64,
    /// current turbo boost pressure if available    
    pub turbo_boost_pressure: f64,
    /// offset from static CG to graphical center    
    pub physics_to_graphics_offset: [f32; 3],
    /// the *physical* steering wheel range    
    pub physical_steering_wheel_range: f32,

    // Future use
    /// for future use (note that the slot ID has been moved to mID above)    
    expansion: [Garbage; 128 + 24],

    // keeping this at the end of the structure to make it easier to replace in future versions
    /// wheel info (front left, front right, rear left, rear right)    
    pub wheels: [PageWheelTelemetry; 4],
}

#[repr(C, packed(4))]
#[derive(Copy, Clone, Debug)]
pub struct PageWheelTelemetry {
    /// meters
    pub suspension_deflection: f64,
    /// meters
    pub ride_height: f64,
    /// pushrod load in Newtons
    pub susp_force: f64,
    /// Celsius
    pub brake_temp: f64,
    /// currently 0.0-1.0, depending on driver input and brake balance; will convert to true brake pressure (kPa) in future
    pub brake_pressure: f64,

    /// radians/sec
    pub rotation: f64,
    /// lateral velocity at contact patch
    pub lateral_patch_vel: f64,
    /// longitudinal velocity at contact patch
    pub longitudinal_patch_vel: f64,
    /// lateral velocity at contact patch
    pub lateral_ground_vel: f64,
    /// longitudinal velocity at contact patch
    pub longitudinal_ground_vel: f64,
    /// radians (positive is left for left-side wheels, right for right-side wheels)
    pub camber: f64,
    /// Newtons
    pub lateral_force: f64,
    /// Newtons
    pub longitudinal_force: f64,
    /// Newtons
    pub tire_load: f64,

    /// an approximation of what fraction of the contact patch is sliding
    pub grip_fract: f64,
    /// kPa (tire pressure)
    pub pressure: f64,
    /// Kelvin (subtract 273.15 to get Celsius), left/center/right (not to be confused with inside/center/outside!)
    pub temperature: [f64; 3],
    /// wear (0.0-1.0, fraction of maximum) ... this is not necessarily proportional with grip loss
    pub wear: f64,
    /// the material prefixes from the TDF file
    pub terrain_name: String16,
    /// Enum for surface type
    pub surface_type: u8,
    /// whether tire is flat
    pub flat: u8,
    /// whether wheel is detached
    pub detached: u8,
    /// tire radius in centimeters
    pub static_undeflected_radius: u8,

    /// how much is tire deflected from its (speed-sensitive) radius
    pub vertical_tire_deflection: f64,
    /// wheel's y location relative to vehicle y location
    pub wheel_ylocation: f64,
    /// current toe angle w.r.t. the vehicle
    pub toe: f64,

    /// rough average of temperature samples from carcass (Kelvin)
    pub tire_carcass_temperature: f64,
    /// rough average of temperature samples from innermost layer of rubber (before carcass) (Kelvin)
    pub tire_inner_layer_temperature: [f64; 3],

    /// for future use
    expansion: [Garbage; 24],
}

#[repr(C, packed(4))]
#[derive(Copy, Clone, Debug)]
pub struct PageScoring {
    pub header: PageHeader,

    /// How many bytes of the structure were written during the last update.
    ///
    /// 0 means unknown (whole buffer should be considered as updated).
    pub bytes_updated_hint: i32,

    pub scoring_info: PageScoringInfo,

    pub vehicles: [PageVehicleScoring; MAX_MAPPED_VEHICLES],
}

#[repr(C, packed(4))]
#[derive(Copy, Clone, Debug)]
pub struct PageScoringInfo {
    /// current track name
    pub track_name: String64,
    /// current session (0=testday 1-4=practice 5-8=qual 9=warmup 10-13=race)
    pub session: i32,
    /// current time
    pub current_et: f64,
    /// ending time
    pub end_et: f64,
    /// maximum laps
    pub max_laps: i32,
    /// distance around track
    pub lap_dist: f64,
    /// results stream additions since last update (newline-delimited and NULL-terminated)
    pointer1: [Garbage; 8],

    /// current number of vehicles
    pub num_vehicles: i32,

    /// Game phases
    pub game_phase: u8,

    /// Yellow flag states (applies to full-course only)
    pub yellow_flag_state: i8,

    /// whether there are any local yellows at the moment in each sector (not sure if sector 0 is first or last, so test)
    pub sector_flag: [i8; 3],
    /// start light frame (number depends on track)
    pub start_light: u8,
    /// number of red lights in start sequence
    pub num_red_lights: u8,
    /// in realtime as opposed to at the monitor
    pub in_realtime: u8,
    /// player name (including possible multiplayer override)
    pub player_name: String32,
    /// may be encoded to be a legal filename
    pub plr_file_name: String64,

    // weather
    /// cloud darkness? 0.0-1.0
    pub dark_cloud: f64,
    /// raining severity 0.0-1.0
    pub raining: f64,
    /// temperature (Celsius)
    pub ambient_temp: f64,
    /// temperature (Celsius)
    pub track_temp: f64,
    /// wind speed
    pub wind: PageVec3,
    /// minimum wetness on main path 0.0-1.0
    pub min_path_wetness: f64,
    /// maximum wetness on main path 0.0-1.0
    pub max_path_wetness: f64,

    // multiplayer
    /// 1 = server, 2 = client, 3 = server and client
    pub game_mode: u8,
    /// is the server password protected
    pub is_password_protected: u8,
    /// the port of the server (if on a server)
    pub server_port: u16,
    /// the public IP address of the server (if on a server)
    pub server_public_ip: u32,
    /// maximum number of vehicles that can be in the session
    pub max_players: i32,
    /// name of the server
    pub server_name: String32,
    /// start time (seconds since midnight) of the event
    pub start_et: f32,

    /// average wetness on main path 0.0-1.0
    pub avg_path_wetness: f64,

    /// Future use
    expansion: [Garbage; 200],

    /// array of vehicle scoring info's
    pointer2: [Garbage; 8],
}

#[repr(C, packed(4))]
#[derive(Copy, Clone, Debug)]
pub struct PageVehicleScoring {
    /// slot ID (note that it can be re-used in multiplayer after someone leaves)
    pub id: i32,
    /// driver name
    pub driver_name: String32,
    /// vehicle name
    pub vehicle_name: String64,
    /// laps completed
    pub total_laps: i16,
    /// Sector
    pub sector: i8,
    /// Finish status
    pub finish_status: i8,
    /// current distance around track
    pub lap_dist: f64,
    /// lateral position with respect to *very approximate* "center" path
    pub path_lateral: f64,
    /// track edge (w.r.t. "center" path) on same side of track as vehicle
    pub track_edge: f64,

    /// best sector 1
    pub best_sector1: f64,
    /// best sector 2 (plus sector 1)
    pub best_sector2: f64,
    /// best lap time
    pub best_lap_time: f64,
    /// last sector 1
    pub last_sector1: f64,
    /// last sector 2 (plus sector 1)
    pub last_sector2: f64,
    /// last lap time
    pub last_lap_time: f64,
    /// current sector 1 if valid
    pub cur_sector1: f64,
    /// current sector 2 (plus sector 1) if valid
    pub cur_sector2: f64,
    // no current laptime because it instantly becomes "last"
    /// number of pitstops made    
    pub num_pitstops: i16,
    /// number of outstanding penalties    
    pub num_penalties: i16,
    /// is this the player's vehicle    
    pub is_player: u8,

    /// Who is in control    
    pub control: i8,
    /// between pit entrance and pit exit (not always accurate for remote vehicles)    
    pub in_pits: u8,
    /// 1-based position    
    pub place: u8,
    /// vehicle class    
    pub vehicle_class: String32,

    // Dash Indicators
    /// time behind vehicle in next higher place    
    pub time_behind_next: f64,
    /// laps behind vehicle in next higher place    
    pub laps_behind_next: i32,
    /// time behind leader    
    pub time_behind_leader: f64,
    /// laps behind leader    
    pub laps_behind_leader: i32,
    /// time this lap was started    
    pub lap_start_et: f64,

    // Position and derivatives
    /// world position in meters    
    pub pos: PageVec3,
    /// velocity (meters/sec) in local vehicle coordinates    
    pub local_vel: PageVec3,
    /// acceleration (meters/sec^2) in local vehicle coordinates    
    pub local_accel: PageVec3,

    // Orientation and derivatives
    /// rows of orientation matrix (use TelemQuat conversions if desired), also converts local    
    pub ori: [PageVec3; 3],
    // vehicle vectors into world X, Y, or Z using dot product of rows 0, 1, or 2 respectively
    /// rotation (radians/sec) in local vehicle coordinates    
    pub local_rot: PageVec3,
    /// rotational acceleration (radians/sec^2) in local vehicle coordinates    
    pub local_rot_accel: PageVec3,

    // tag.2012.03.01 - stopped casting some of these so variables now have names and mExpansion has shrunk, overall size and old data locations should be same
    /// status of headlights    
    pub headlights: u8,
    pub pit_state: u8,
    /// whether this vehicle is being scored by server (could be off in qualifying or racing heats)    
    pub server_scored: u8,
    /// game phases (described below) plus 9=after formation, 10=under yellow, 11=under blue (not used)    
    pub individual_phase: u8,

    /// 1-based, can be -1 when invalid    
    pub qualification: i32,

    /// estimated time into lap    
    pub time_into_lap: f64,
    /// estimated laptime used for 'time behind' and 'time into lap' (note: this may changed based on vehicle and setup!?)    
    pub estimated_lap_time: f64,

    /// pit group (same as team name unless pit is shared)    
    pub pit_group: String24,
    /// primary flag being shown to vehicle    
    pub flag: u8,
    /// whether this car has taken a full-course caution flag at the start/finish line    
    pub under_yellow: u8,
    pub count_lap_flag: u8,
    /// appears to be within the correct garage stall    
    pub in_garage_stall: u8,

    /// Coded upgrades    
    pub upgrade_pack: String16,

    /// location of pit in terms of lap distance    
    pub pit_lap_dist: f32,

    /// sector 1 time from best lap (not necessarily the best sector 1 time)    
    pub best_lap_sector1: f32,
    /// sector 2 time from best lap (not necessarily the best sector 2 time)    
    pub best_lap_sector2: f32,

    /// for future use    
    expansion: [Garbage; 48],
}

#[repr(C, packed(4))]
#[derive(Copy, Clone, Debug)]
pub struct PageRules {
    pub header: PageHeader,

    /// How many bytes of the structure were written during the last update.
    ///
    /// 0 means unknown (whole buffer should be considered as updated).
    pub bytes_updated_hint: i32,
}

#[repr(C, packed(4))]
#[derive(Copy, Clone, Debug)]
pub struct PageMultiRules {
    pub header: PageHeader,

    /// How many bytes of the structure were written during the last update.
    ///
    /// 0 means unknown (whole buffer should be considered as updated).
    pub bytes_updated_hint: i32,
}

#[repr(C, packed(4))]
#[derive(Copy, Clone, Debug)]
pub struct PagePitInfo {
    pub header: PageHeader,
}

#[repr(C, packed(4))]
#[derive(Copy, Clone, Debug)]
pub struct PageWeather {
    pub header: PageHeader,
}

#[repr(C, packed(4))]
#[derive(Copy, Clone, Debug)]
pub struct PageExtended {
    pub header: PageHeader,
}