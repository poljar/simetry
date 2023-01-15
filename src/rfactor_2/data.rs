use crate::rfactor_2::shared_memory_data::{
    PageExtended, PageForceFeedback, PageHeader, PageMultiRules, PagePitInfo, PageRules,
    PageScoring, PageScoringInfo, PageTelemetry, PageVec3, PageVehicleScoring,
    PageVehicleTelemetry, PageWeather, PageWheelTelemetry, MAX_MAPPED_VEHICLES,
};
use crate::windows_util::cp1252_to_string;
use anyhow::{bail, Error, Result};

#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct PacketId(pub u32);

#[derive(Clone, Debug, Default)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
}

#[derive(Clone, Debug)]
pub struct ForceFeedback {
    /// Current FFB value
    pub force_value: f64,
}

#[derive(Clone, Debug)]
pub struct Telemetry {
    pub packet_id: PacketId,
    pub vehicles: Vec<VehicleTelemetry>,
}

#[derive(Clone, Debug)]
pub struct VehicleTelemetry {
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
    pub vehicle_name: String,
    /// current track name    
    pub track_name: String,

    // Position and derivatives
    /// world position in meters    
    pub pos: Vec3,
    /// velocity (meters/sec) in local vehicle coordinates    
    pub local_vel: Vec3,
    /// acceleration (meters/sec^2) in local vehicle coordinates    
    pub local_accel: Vec3,

    // Orientation and derivatives
    /// rows of orientation matrix (use TelemQuat conversions if desired), also converts local    
    pub ori: [Vec3; 3],
    // vehicle vectors into world X, Y, or Z using dot product of rows 0, 1, or 2 respectively
    /// rotation (radians/sec) in local vehicle coordinates    
    pub local_rot: Vec3,
    /// rotational acceleration (radians/sec^2) in local vehicle coordinates    
    pub local_rot_accel: Vec3,

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
    pub last_impact_pos: Vec3,

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
    pub front_tire_compound_name: String,
    /// name of rear tire compound    
    pub rear_tire_compound_name: String,

    /// whether speed limiter is available    
    pub speed_limiter_available: u8,
    /// whether (hard) anti-stall is activated    
    pub anti_stall_activated: u8,

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

    // keeping this at the end of the structure to make it easier to replace in future versions
    /// wheel info (front left, front right, rear left, rear right)    
    pub wheels: [WheelTelemetry; 4],
}

#[derive(Clone, Debug)]
pub struct WheelTelemetry {
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
    pub terrain_name: String,
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
}

#[derive(Clone, Debug)]
pub struct Scoring {
    pub packet_id: PacketId,

    pub scoring_info: ScoringInfo,

    pub vehicles: Vec<VehicleScoring>,
}

#[derive(Clone, Debug)]
pub struct ScoringInfo {
    /// current track name
    pub track_name: String,
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
    pub player_name: String,
    /// may be encoded to be a legal filename
    pub plr_file_name: String,

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
    pub wind: Vec3,
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
    pub server_name: String,
    /// start time (seconds since midnight) of the event
    pub start_et: f32,

    /// average wetness on main path 0.0-1.0
    pub avg_path_wetness: f64,
}

#[derive(Clone, Debug)]
pub struct VehicleScoring {
    /// slot ID (note that it can be re-used in multiplayer after someone leaves)
    pub id: i32,
    /// driver name
    pub driver_name: String,
    /// vehicle name
    pub vehicle_name: String,
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
    pub vehicle_class: String,

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
    pub pos: Vec3,
    /// velocity (meters/sec) in local vehicle coordinates    
    pub local_vel: Vec3,
    /// acceleration (meters/sec^2) in local vehicle coordinates    
    pub local_accel: Vec3,

    // Orientation and derivatives
    /// rows of orientation matrix (use TelemQuat conversions if desired), also converts local    
    pub ori: [Vec3; 3],
    // vehicle vectors into world X, Y, or Z using dot product of rows 0, 1, or 2 respectively
    /// rotation (radians/sec) in local vehicle coordinates    
    pub local_rot: Vec3,
    /// rotational acceleration (radians/sec^2) in local vehicle coordinates    
    pub local_rot_accel: Vec3,

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
    pub pit_group: String,
    /// primary flag being shown to vehicle    
    pub flag: u8,
    /// whether this car has taken a full-course caution flag at the start/finish line    
    pub under_yellow: u8,
    pub count_lap_flag: u8,
    /// appears to be within the correct garage stall    
    pub in_garage_stall: u8,

    /// Coded upgrades    
    pub upgrade_pack: String,

    /// location of pit in terms of lap distance    
    pub pit_lap_dist: f32,

    /// sector 1 time from best lap (not necessarily the best sector 1 time)    
    pub best_lap_sector1: f32,
    /// sector 2 time from best lap (not necessarily the best sector 2 time)    
    pub best_lap_sector2: f32,
}

#[derive(Clone, Debug)]
pub struct Rules {
    pub packet_id: PacketId,
}

#[derive(Clone, Debug)]
pub struct MultiRules {
    pub packet_id: PacketId,
}

#[derive(Clone, Debug)]
pub struct PitInfo {
    pub packet_id: PacketId,
}

#[derive(Clone, Debug)]
pub struct Weather {
    pub packet_id: PacketId,
}

#[derive(Clone, Debug)]
pub struct Extended {
    pub packet_id: PacketId,
}

impl TryFrom<PageHeader> for PacketId {
    type Error = Error;

    fn try_from(header: PageHeader) -> Result<Self> {
        if header.version_update_begin != header.version_update_end {
            bail!("Page was in the process of being written")
        }
        Ok(PacketId(header.version_update_end))
    }
}

impl TryFrom<Box<PageForceFeedback>> for ForceFeedback {
    type Error = Error;

    fn try_from(value: Box<PageForceFeedback>) -> Result<Self> {
        let _packet_id: PacketId = value.ignored_header.try_into()?;
        Ok(Self {
            force_value: value.force_value,
        })
    }
}

impl TryFrom<Box<PageTelemetry>> for Telemetry {
    type Error = Error;

    fn try_from(value: Box<PageTelemetry>) -> Result<Telemetry> {
        let packet_id = value.header.try_into()?;
        let vehicle_count = value.num_vehicles.clamp(0, MAX_MAPPED_VEHICLES as i32) as usize;
        Ok(Self {
            packet_id,
            vehicles: value
                .vehicles
                .iter()
                .take(vehicle_count)
                .map(|v| v.into())
                .collect(),
        })
    }
}

impl From<&PageVehicleTelemetry> for VehicleTelemetry {
    fn from(value: &PageVehicleTelemetry) -> Self {
        Self {
            id: value.id,
            delta_time: value.delta_time,
            elapsed_time: value.elapsed_time,
            lap_number: value.lap_number,
            lap_start_et: value.lap_start_et,
            vehicle_name: cp1252_to_string(&value.vehicle_name).unwrap_or_default(),
            track_name: cp1252_to_string(&value.track_name).unwrap_or_default(),
            pos: value.pos.into(),
            local_vel: value.local_vel.into(),
            local_accel: value.local_accel.into(),
            ori: value.ori.map(Into::into),
            local_rot: value.local_rot.into(),
            local_rot_accel: value.local_rot_accel.into(),
            gear: value.gear,
            engine_rpm: value.engine_rpm,
            engine_water_temp: value.engine_water_temp,
            engine_oil_temp: value.engine_oil_temp,
            clutch_rpm: value.clutch_rpm,
            unfiltered_throttle: value.unfiltered_throttle,
            unfiltered_brake: value.unfiltered_brake,
            unfiltered_steering: value.unfiltered_steering,
            unfiltered_clutch: value.unfiltered_clutch,
            filtered_throttle: value.filtered_throttle,
            filtered_brake: value.filtered_brake,
            filtered_steering: value.filtered_steering,
            filtered_clutch: value.filtered_clutch,
            steering_shaft_torque: value.steering_shaft_torque,
            front3rd_deflection: value.front3rd_deflection,
            rear3rd_deflection: value.rear3rd_deflection,
            front_wing_height: value.front_wing_height,
            front_ride_height: value.front_ride_height,
            rear_ride_height: value.rear_ride_height,
            drag: value.drag,
            front_downforce: value.front_downforce,
            rear_downforce: value.rear_downforce,
            fuel: value.fuel,
            engine_max_rpm: value.engine_max_rpm,
            scheduled_stops: value.scheduled_stops,
            overheating: value.overheating,
            detached: value.detached,
            headlights: value.headlights,
            dent_severity: value.dent_severity,
            last_impact_et: value.last_impact_et,
            last_impact_magnitude: value.last_impact_magnitude,
            last_impact_pos: value.last_impact_pos.into(),
            engine_torque: value.engine_torque,
            current_sector: value.current_sector,
            speed_limiter: value.speed_limiter,
            max_gears: value.max_gears,
            front_tire_compound_index: value.front_tire_compound_index,
            rear_tire_compound_index: value.rear_tire_compound_index,
            fuel_capacity: value.fuel_capacity,
            front_flap_activated: value.front_flap_activated,
            rear_flap_activated: value.rear_flap_activated,
            rear_flap_legal_status: value.rear_flap_legal_status,
            ignition_starter: value.ignition_starter,
            front_tire_compound_name: cp1252_to_string(&value.front_tire_compound_name)
                .unwrap_or_default(),
            rear_tire_compound_name: cp1252_to_string(&value.rear_tire_compound_name)
                .unwrap_or_default(),
            speed_limiter_available: value.speed_limiter_available,
            anti_stall_activated: value.anti_stall_activated,
            visual_steering_wheel_range: value.visual_steering_wheel_range,
            rear_brake_bias: value.rear_brake_bias,
            turbo_boost_pressure: value.turbo_boost_pressure,
            physics_to_graphics_offset: value.physics_to_graphics_offset,
            physical_steering_wheel_range: value.physical_steering_wheel_range,
            wheels: value.wheels.map(|v| (&v).into()),
        }
    }
}

impl From<&PageWheelTelemetry> for WheelTelemetry {
    fn from(value: &PageWheelTelemetry) -> Self {
        Self {
            suspension_deflection: value.suspension_deflection,
            ride_height: value.ride_height,
            susp_force: value.susp_force,
            brake_temp: value.brake_temp,
            brake_pressure: value.brake_pressure,
            rotation: value.rotation,
            lateral_patch_vel: value.lateral_patch_vel,
            longitudinal_patch_vel: value.longitudinal_patch_vel,
            lateral_ground_vel: value.lateral_ground_vel,
            longitudinal_ground_vel: value.longitudinal_ground_vel,
            camber: value.camber,
            lateral_force: value.lateral_force,
            longitudinal_force: value.longitudinal_force,
            tire_load: value.tire_load,
            grip_fract: value.grip_fract,
            pressure: value.pressure,
            temperature: value.temperature,
            wear: value.wear,
            terrain_name: cp1252_to_string(&value.terrain_name).unwrap_or_default(),
            surface_type: value.surface_type,
            flat: value.flat,
            detached: value.detached,
            static_undeflected_radius: value.static_undeflected_radius,
            vertical_tire_deflection: value.vertical_tire_deflection,
            wheel_ylocation: value.wheel_ylocation,
            toe: value.toe,
            tire_carcass_temperature: value.tire_carcass_temperature,
            tire_inner_layer_temperature: value.tire_inner_layer_temperature,
        }
    }
}

impl TryFrom<Box<PageScoring>> for Scoring {
    type Error = Error;

    fn try_from(value: Box<PageScoring>) -> Result<Scoring> {
        let packet_id = value.header.try_into()?;
        let scoring_info: ScoringInfo = (&value.scoring_info).into();
        let vehicles = value
            .vehicles
            .iter()
            .take(
                scoring_info
                    .num_vehicles
                    .clamp(0, MAX_MAPPED_VEHICLES as i32) as usize,
            )
            .map(Into::into)
            .collect();
        Ok(Self {
            packet_id,
            scoring_info,
            vehicles,
        })
    }
}

impl From<&PageScoringInfo> for ScoringInfo {
    fn from(value: &PageScoringInfo) -> Self {
        Self {
            track_name: cp1252_to_string(&value.track_name).unwrap_or_default(),
            session: value.session,
            current_et: value.current_et,
            end_et: value.end_et,
            max_laps: value.max_laps,
            lap_dist: value.lap_dist,
            num_vehicles: value.num_vehicles,
            game_phase: value.game_phase,
            yellow_flag_state: value.yellow_flag_state,
            sector_flag: value.sector_flag,
            start_light: value.start_light,
            num_red_lights: value.num_red_lights,
            in_realtime: value.in_realtime,
            player_name: cp1252_to_string(&value.player_name).unwrap_or_default(),
            plr_file_name: cp1252_to_string(&value.plr_file_name).unwrap_or_default(),
            dark_cloud: value.dark_cloud,
            raining: value.raining,
            ambient_temp: value.ambient_temp,
            track_temp: value.track_temp,
            wind: value.wind.into(),
            min_path_wetness: value.min_path_wetness,
            max_path_wetness: value.max_path_wetness,
            game_mode: value.game_mode,
            is_password_protected: value.is_password_protected,
            server_port: value.server_port,
            server_public_ip: value.server_public_ip,
            max_players: value.max_players,
            server_name: cp1252_to_string(&value.server_name).unwrap_or_default(),
            start_et: value.start_et,
            avg_path_wetness: value.avg_path_wetness,
        }
    }
}

impl From<&PageVehicleScoring> for VehicleScoring {
    fn from(value: &PageVehicleScoring) -> Self {
        Self {
            id: value.id,
            driver_name: cp1252_to_string(&value.driver_name).unwrap_or_default(),
            vehicle_name: cp1252_to_string(&value.vehicle_name).unwrap_or_default(),
            total_laps: value.total_laps,
            sector: value.sector,
            finish_status: value.finish_status,
            lap_dist: value.lap_dist,
            path_lateral: value.path_lateral,
            track_edge: value.track_edge,
            best_sector1: value.best_sector1,
            best_sector2: value.best_sector2,
            best_lap_time: value.best_lap_time,
            last_sector1: value.last_sector1,
            last_sector2: value.last_sector2,
            last_lap_time: value.last_lap_time,
            cur_sector1: value.cur_sector1,
            cur_sector2: value.cur_sector2,
            num_pitstops: value.num_pitstops,
            num_penalties: value.num_penalties,
            is_player: value.is_player,
            control: value.control,
            in_pits: value.in_pits,
            place: value.place,
            vehicle_class: cp1252_to_string(&value.vehicle_class).unwrap_or_default(),
            time_behind_next: value.time_behind_next,
            laps_behind_next: value.laps_behind_next,
            time_behind_leader: value.time_behind_leader,
            laps_behind_leader: value.laps_behind_leader,
            lap_start_et: value.lap_start_et,
            pos: Default::default(),
            local_vel: Default::default(),
            local_accel: Default::default(),
            ori: value.ori.map(Into::into),
            local_rot: Default::default(),
            local_rot_accel: Default::default(),
            headlights: value.headlights,
            pit_state: value.pit_state,
            server_scored: value.server_scored,
            individual_phase: value.individual_phase,
            qualification: value.qualification,
            time_into_lap: value.time_into_lap,
            estimated_lap_time: value.estimated_lap_time,
            pit_group: cp1252_to_string(&value.pit_group).unwrap_or_default(),
            flag: value.flag,
            under_yellow: value.under_yellow,
            count_lap_flag: value.count_lap_flag,
            in_garage_stall: value.in_garage_stall,
            upgrade_pack: cp1252_to_string(&value.upgrade_pack).unwrap_or_default(),
            pit_lap_dist: value.pit_lap_dist,
            best_lap_sector1: value.best_lap_sector1,
            best_lap_sector2: value.best_lap_sector2,
        }
    }
}

impl TryFrom<Box<PageRules>> for Rules {
    type Error = Error;

    fn try_from(value: Box<PageRules>) -> Result<Rules> {
        let packet_id = value.header.try_into()?;
        Ok(Self { packet_id })
    }
}

impl TryFrom<Box<PageMultiRules>> for MultiRules {
    type Error = Error;

    fn try_from(value: Box<PageMultiRules>) -> Result<MultiRules> {
        let packet_id = value.header.try_into()?;
        Ok(Self { packet_id })
    }
}

impl TryFrom<Box<PagePitInfo>> for PitInfo {
    type Error = Error;

    fn try_from(value: Box<PagePitInfo>) -> Result<PitInfo> {
        let packet_id = value.header.try_into()?;
        Ok(Self { packet_id })
    }
}

impl TryFrom<Box<PageWeather>> for Weather {
    type Error = Error;

    fn try_from(value: Box<PageWeather>) -> Result<Weather> {
        let packet_id = value.header.try_into()?;
        Ok(Self { packet_id })
    }
}

impl TryFrom<Box<PageExtended>> for Extended {
    type Error = Error;

    fn try_from(value: Box<PageExtended>) -> Result<Extended> {
        let packet_id = value.header.try_into()?;
        Ok(Self { packet_id })
    }
}

impl From<PageVec3> for Vec3 {
    fn from(value: PageVec3) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}