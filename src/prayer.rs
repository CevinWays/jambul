use serde::Serialize;

const FAJR_ANGLE_DEGREES: f64 = 20.0;
const ISHA_ANGLE_DEGREES: f64 = 18.0;
const HORIZON_ANGLE_DEGREES: f64 = 0.833;
const IHTIYATI_MINUTES: f64 = 2.0;
const MINUTES_PER_HOUR: f64 = 60.0;
const DEGREES_PER_HOUR: f64 = 15.0;
const J2000_JULIAN_DAY: f64 = 2_451_545.0;

#[derive(Serialize, Debug, PartialEq)]
pub struct PrayerTimes {
    pub fajr: f64,
    pub sunrise: f64,
    pub dhuhr: f64,
    pub asr: f64,
    pub maghrib: f64,
    pub isha: f64,
}

struct Sun {
    declination: f64,
    equation_of_time_hours: f64,
}

fn julian_day(year: i32, month: u32, day: u32) -> f64 {
    let (year, month) = if month <= 2 { (year - 1, month + 12) } else { (year, month) };
    let century = (year as f64 / 100.0).floor();
    let gregorian_correction = 2.0 - century + (century / 4.0).floor();
    (365.25 * (year as f64 + 4716.0)).floor()
        + (30.6001 * (month as f64 + 1.0)).floor()
        + day as f64
        + gregorian_correction
        - 1524.5
}

fn sun_at(julian_day: f64) -> Sun {
    let days = julian_day - J2000_JULIAN_DAY;
    let mean_anomaly = (357.529 + 0.985_600_28 * days).rem_euclid(360.0).to_radians();
    let mean_longitude = (280.459 + 0.985_647_36 * days).rem_euclid(360.0);
    let ecliptic_longitude = (mean_longitude + 1.915 * mean_anomaly.sin() + 0.020 * (2.0 * mean_anomaly).sin())
        .rem_euclid(360.0)
        .to_radians();
    let obliquity = (23.439 - 0.000_000_36 * days).to_radians();
    let right_ascension_hours = (obliquity.cos() * ecliptic_longitude.sin())
        .atan2(ecliptic_longitude.cos())
        .to_degrees()
        .rem_euclid(360.0)
        / DEGREES_PER_HOUR;
    Sun {
        declination: (obliquity.sin() * ecliptic_longitude.sin()).asin(),
        equation_of_time_hours: mean_longitude / DEGREES_PER_HOUR - right_ascension_hours,
    }
}

fn hours_from_noon(angle_below_horizon_degrees: f64, latitude: f64, declination: f64) -> Option<f64> {
    let ratio = (-angle_below_horizon_degrees.to_radians().sin() - declination.sin() * latitude.sin())
        / (declination.cos() * latitude.cos());
    (-1.0..=1.0).contains(&ratio).then(|| ratio.acos().to_degrees() / DEGREES_PER_HOUR)
}

fn asr_angle_degrees(latitude: f64, declination: f64) -> f64 {
    let shadow_length = 1.0 + (latitude - declination).abs().tan();
    -(1.0 / shadow_length).atan().to_degrees()
}

// shortcut: sun position is taken at local noon for every time, about a minute of error, the ihtiyati margin absorbs it
pub fn prayer_times(
    latitude_degrees: f64,
    longitude_degrees: f64,
    year: i32,
    month: u32,
    day: u32,
    utc_offset_minutes: i32,
) -> Option<PrayerTimes> {
    let latitude = latitude_degrees.to_radians();
    let utc_offset_hours = utc_offset_minutes as f64 / MINUTES_PER_HOUR;
    let julian = julian_day(year, month, day) - longitude_degrees / (DEGREES_PER_HOUR * 24.0);
    let sun = sun_at(julian);
    let noon_hours = 12.0 - sun.equation_of_time_hours + utc_offset_hours - longitude_degrees / DEGREES_PER_HOUR;
    let before = |angle: f64| hours_from_noon(angle, latitude, sun.declination).map(|offset| noon_hours - offset);
    let after = |angle: f64| hours_from_noon(angle, latitude, sun.declination).map(|offset| noon_hours + offset);
    let to_minutes = |hours: f64, adjustment: f64| hours * MINUTES_PER_HOUR + adjustment;
    Some(PrayerTimes {
        fajr: to_minutes(before(FAJR_ANGLE_DEGREES)?, IHTIYATI_MINUTES),
        // TODO: unverified — Kemenag's sunrise ihtiyati is assumed to be subtracted
        sunrise: to_minutes(before(HORIZON_ANGLE_DEGREES)?, -IHTIYATI_MINUTES),
        dhuhr: to_minutes(noon_hours, IHTIYATI_MINUTES),
        asr: to_minutes(after(asr_angle_degrees(latitude, sun.declination))?, IHTIYATI_MINUTES),
        maghrib: to_minutes(after(HORIZON_ANGLE_DEGREES)?, IHTIYATI_MINUTES),
        isha: to_minutes(after(ISHA_ANGLE_DEGREES)?, IHTIYATI_MINUTES),
    })
}

#[tauri::command]
pub async fn prayer_times_for_day(
    latitude: f64,
    longitude: f64,
    year: i32,
    month: u32,
    day: u32,
    utc_offset_minutes: i32,
) -> Option<PrayerTimes> {
    prayer_times(latitude, longitude, year, month, day, utc_offset_minutes)
}

#[cfg(test)]
mod tests {
    use super::*;

    const JAKARTA_LATITUDE: f64 = -6.2;
    const JAKARTA_LONGITUDE: f64 = 106.8;
    const WIB_OFFSET_MINUTES: i32 = 420;

    fn jakarta_on(year: i32, month: u32, day: u32) -> PrayerTimes {
        prayer_times(JAKARTA_LATITUDE, JAKARTA_LONGITUDE, year, month, day, WIB_OFFSET_MINUTES).unwrap()
    }

    fn assert_near(actual_minutes: f64, hour: u32, minute: u32) {
        let expected = (hour * 60 + minute) as f64;
        assert!((actual_minutes - expected).abs() <= 4.0, "{actual_minutes} is not near {hour}:{minute}");
    }

    #[test]
    fn jakarta_late_november() {
        let times = jakarta_on(2026, 11, 24);
        assert_near(times.fajr, 4, 5);
        assert_near(times.sunrise, 5, 24);
        assert_near(times.dhuhr, 11, 42);
        assert_near(times.asr, 15, 7);
        assert_near(times.maghrib, 17, 55);
        assert_near(times.isha, 19, 6);
    }

    #[test]
    fn times_are_in_order() {
        let times = jakarta_on(2026, 6, 21);
        assert!(times.fajr < times.sunrise && times.sunrise < times.dhuhr);
        assert!(times.dhuhr < times.asr && times.asr < times.maghrib && times.maghrib < times.isha);
    }

    #[test]
    fn no_fajr_inside_the_polar_summer() {
        assert_eq!(prayer_times(78.0, 15.0, 2026, 6, 21, 120), None);
    }
}
