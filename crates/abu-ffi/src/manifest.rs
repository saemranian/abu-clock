//! Synchronized Session Manifest Exporter (`.abuscan.json` & Colmap `images.txt`)
//! for RealityCapture, Colmap, LixelCyberColor, and Jawset 3D Gaussian Splatting (3DGS).

use serde::{Deserialize, Serialize};

/// Single synchronized multi-camera or LiDAR capture record in an Abu Scan session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AbuScanCaptureRecord {
    pub image_id: u32,
    pub sensor_id: String,
    pub file_path: String,
    pub smpte_timecode: String,
    pub bcd_timecode: u32,
    pub tai_ns: u64,
    pub error_ns: i64,
    pub qw: f64,
    pub qx: f64,
    pub qy: f64,
    pub qz: f64,
    pub tx_m: f64,
    pub ty_m: f64,
    pub tz_m: f64,
    pub camera_model_id: u32,
}

/// Complete Abu Scan session manifest aggregating multi-camera and RoboSense E1R / XGRIDS L2PRO captures.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AbuScanSessionManifest {
    pub schema: String,
    pub session_name: String,
    pub captures: Vec<AbuScanCaptureRecord>,
}

impl AbuScanSessionManifest {
    /// Creates a new empty `AbuScanSessionManifest`.
    pub fn new(session_name: impl Into<String>) -> Self {
        Self {
            schema: "abuscan.manifest.v1".to_string(),
            session_name: session_name.into(),
            captures: Vec::new(),
        }
    }

    /// Appends a synchronized capture record.
    pub fn push_record(&mut self, record: AbuScanCaptureRecord) {
        self.captures.push(record);
    }

    /// Serializes the session manifest as pretty-printed `.abuscan.json`.
    pub fn to_abuscan_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("valid AbuScanSessionManifest JSON")
    }

    /// Exports the session manifest in standard Colmap / Jawset Postshot `images.txt` format:
    /// `IMAGE_ID QW QX QY QZ TX TY TZ CAMERA_ID NAME` followed by an empty `POINTS2D` line.
    pub fn to_colmap_images_txt(&self) -> String {
        let mut out = String::from(
            "# Image list with two lines of data per image:\n\
             #   IMAGE_ID, QW, QX, QY, QZ, TX, TY, TZ, CAMERA_ID, NAME\n\
             #   POINTS2D[] as (X, Y, POINT3D_ID)\n",
        );
        for rec in &self.captures {
            out.push_str(&format!(
                "{} {:.6} {:.6} {:.6} {:.6} {:.6} {:.6} {:.6} {} {}\n\n",
                rec.image_id,
                rec.qw,
                rec.qx,
                rec.qy,
                rec.qz,
                rec.tx_m,
                rec.ty_m,
                rec.tz_m,
                rec.camera_model_id,
                rec.file_path
            ));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abuscan_json_and_colmap_images_txt_export_matches_spec() {
        let mut manifest = AbuScanSessionManifest::new("StageA_Take01_3DGS");
        manifest.push_record(AbuScanCaptureRecord {
            image_id: 1,
            sensor_id: "iPhone16Pro_CamA".to_string(),
            file_path: "iPhone16Pro_CamA_14221008_1729180800050000000.dng".to_string(),
            smpte_timecode: "14:22:10;08".to_string(),
            bcd_timecode: 0x1422_1008,
            tai_ns: 1_729_180_800_050_000_000,
            error_ns: 38,
            qw: 1.0,
            qx: 0.0,
            qy: 0.0,
            qz: 0.0,
            tx_m: 1.25,
            ty_m: -0.40,
            tz_m: 1.70,
            camera_model_id: 1,
        });

        let json_str = manifest.to_abuscan_json();
        assert!(json_str.contains("\"abuscan.manifest.v1\""));
        assert!(json_str.contains("\"error_ns\": 38"));

        let colmap_txt = manifest.to_colmap_images_txt();
        assert!(colmap_txt.contains(
            "1 1.000000 0.000000 0.000000 0.000000 1.250000 -0.400000 1.700000 1 iPhone16Pro_CamA_14221008_1729180800050000000.dng"
        ));
    }
}
