use chrono::{NaiveDate, NaiveTime, TimeZone, Utc};
use serialport::{self, DataBits, FlowControl, Parity, StopBits};
use std::io::{Read, Write};
use std::process::exit;
use std::time::Duration;

#[derive(Debug)]
#[allow(dead_code)] //we allow dead code because it's there to be used later when expanding
enum ParseResult {
    Date(NaiveDate),
    Time(NaiveTime),
    KWh(u64),
    UnitlessNumeric(u64),
    None,
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure the serial port
    let port_name = "/dev/ttyUSB0";
    let mut port;
    let port_result = serialport::new(port_name, 300)
        .data_bits(DataBits::Seven)
        .parity(Parity::Even)
        .stop_bits(StopBits::One)
        .flow_control(FlowControl::None)
        .timeout(Duration::from_secs(100))
        .open();

    match port_result {
        Ok(p) => port = p,
        Err(e) => {
            println!("Failed to open serial port: {:?}", e);
            exit(-1);
        }
    }

    // Send the initial request command to the meter
    println!("Sending control sequence...");
    let request_sequence = "/?!\r\n";
    port.write_all(request_sequence.as_bytes())
        .expect("Failed to write to serial port");

    // Buffer to store the incoming data
    let mut buffer: Vec<u8> = vec![0; 1024];
    let mut collected_data = Vec::new();

    loop {
        // Read data from the serial port
        match port.read(&mut buffer) {
            Ok(n) => {
                // Append the read data to the collected data
                collected_data.extend_from_slice(&buffer[..n]);

                // Print raw data received for debugging
                //println!("Received raw data: {:?}", &buffer[..n]);

                // Check for complete message
                if collected_data.ends_with(b"\r\n") {

                    // Parse the complete message
                    let result = parse_code(&collected_data);
                    println!("parsed: {:?}", result);

                    // Clear the collected data for next read
                    collected_data.clear();
                }
            }
            Err(e) => {
                eprintln!("Error reading from serial port: {:?}", e);
                break;
            }
        }
    }

    Ok(())
}
fn extract_kwh(content: &str) -> Option<u64> {
    let x: Vec<&str> = content.split('*').collect();
    println!("extracting kwh {:?}", x);
    let x1 = x.to_vec().first().unwrap().trim().trim_start_matches('0');
    println!("parse kwh: {:?}", x1);
    if x1.len() == 0 {
        return None;
    } else {
        Some(x1.parse::<u64>().unwrap())
    }
}
/// parse the codes read from the meter
fn parse_code(data: &[u8]) -> ParseResult {
    if data.contains(&"(".as_bytes()[0]) {
        let end_of_obis_identifier = data.iter().position(|&r| r == "(".as_bytes()[0]).unwrap();
        let data_vector = data.to_vec();
        let code = &data_vector[..end_of_obis_identifier];
        let content = &data_vector[end_of_obis_identifier + 1..data.len() - 1];
        println!("code: {:?}, content: {:?}", code, content);

        let vec = content.to_ascii_lowercase();
        let result = String::from_utf8(vec);
        let string = result.unwrap();
        let content_string = string.as_str();
        //we handle the types of messages differently
        let parsed_result = match String::from_utf8(code.to_ascii_lowercase())
            .unwrap()
            .as_str()
        {
            "8.1" | "8.2" | "8.1.1" | "8.2.1" | "8.1.2" | "8.2.2" | "8.0" | "8.0.0" => {
                ParseResult::KWh(extract_kwh(&content_string).or(Some(0)).unwrap())
            }
            "11" => ParseResult::Time(extract_time(&content_string).unwrap()),
            "12" => ParseResult::Date(extract_date(&content_string).unwrap()),
            "12.0.2" | "12.1.1" | "12.1.2" => ParseResult::UnitlessNumeric(
                extract_unitless_numeric(&content_string)
                    .unwrap()
                    .parse()
                    .unwrap(),
            ),
            _ => ParseResult::None,
        };
        return parsed_result;
    }
    ParseResult::None
}
fn extract_unitless_numeric(content: &str) -> Option<&str> {
    Some(&content[..content.len() - 3])
}
fn extract_time(content: &str) -> Option<NaiveTime> {
    let time_components: Vec<&str> = content.split(":").collect(); //hms
    if time_components.len() != 3 {
        return None;
    }
    Some(
        NaiveTime::from_hms_opt(
            time_components[0].parse().unwrap(),
            time_components[0].parse().unwrap(),
            time_components[0].parse().unwrap(),
        )
        .unwrap(),
    )
}
fn extract_date(content: &str) -> Option<NaiveDate> {
    let time_components: Vec<&str> = content[..content.len() - 3].split("-").collect(); //hms
    if time_components.len() != 3 {
        return None;
    }
    let mut year = "20".to_owned();
    year.push_str(time_components[0]);
    Some(
        Utc.with_ymd_and_hms(
            year.parse().unwrap(),
            time_components[1].parse().unwrap(),
            time_components[2].parse().unwrap(),
            12,
            00,
            00,
        )
        .unwrap()
        .date_naive(),
    )
}
