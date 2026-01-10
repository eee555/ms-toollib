use crate::{PyGameBoard, PyVideoActionStateRecorder};
use ms_toollib_original;
use ms_toollib_original::videos::{NewSomeVideo, NewSomeVideo2};
use ms_toollib_original::{GameBoardState, MouseState};
use pyo3::prelude::*;

// 定义宏，生成所有类型录像的子类
macro_rules! generate_video {
    ($name:ident) => {
        #[pyclass(subclass, unsendable)]
        pub struct $name {
            pub core: ms_toollib_original::$name,
        }

        #[pymethods]
        impl $name {
            #[new]
            #[pyo3(signature = (file_name="", raw_data=vec![]))]
            pub fn new(file_name: &str, raw_data: Vec<u8>) -> $name {
                if raw_data.is_empty() {
                    let c = <ms_toollib_original::$name as NewSomeVideo<&str>>::new(file_name);
                    return $name { core: c };
                } else {
                    let c = <ms_toollib_original::$name as NewSomeVideo2<Vec<u8>, &str>>::new(
                        raw_data, file_name,
                    );
                    return $name { core: c };
                }
            }
            pub fn parse(&mut self) {
                self.core.parse().unwrap();
            }
            pub fn analyse(&mut self) {
                self.core.data.analyse();
            }
            pub fn analyse_for_features(&mut self, controller: Vec<String>) {
                let controller_slice: &Vec<&str> =
                    &controller.iter().map(|s| s.as_str()).collect::<Vec<_>>();
                self.core.data.analyse_for_features(controller_slice);
            }
            pub fn generate_evf_v0_raw_data(&mut self) {
                self.core.data.generate_evf_v0_raw_data();
            }
            pub fn generate_evf_v2_raw_data(&mut self) {
                self.core.data.generate_evf_v2_raw_data();
            }
            pub fn generate_evf_v3_raw_data(&mut self) {
                self.core.data.generate_evf_v3_raw_data();
            }
            pub fn generate_evf_v4_raw_data(&mut self) {
                self.core.data.generate_evf_v4_raw_data();
            }
            pub fn save_to_evf_file(&self, file_name: &str) -> PyResult<String> {
                let output_file_name = self.core.data.save_to_evf_file(file_name);
                Ok(output_file_name)
            }
            #[getter]
            fn get_file_name(&self) -> PyResult<String> {
                Ok(self.core.file_name.clone())
            }
            #[getter]
            fn get_raw_data(&self) -> PyResult<Vec<u8>> {
                Ok(self.core.data.get_raw_data().unwrap())
            }
            #[getter]
            fn get_time(&self) -> PyResult<f64> {
                Ok(self.core.data.get_time())
            }
            #[getter]
            fn get_software(&self) -> PyResult<String> {
                Ok(self.core.data.software.clone())
            }
            #[getter]
            fn get_row(&self) -> PyResult<usize> {
                Ok(self.core.data.height)
            }
            #[getter]
            fn get_column(&self) -> PyResult<usize> {
                Ok(self.core.data.width)
            }
            #[getter]
            fn get_level(&self) -> PyResult<u8> {
                Ok(self.core.data.level)
            }
            #[getter]
            fn get_mode(&self) -> PyResult<u16> {
                Ok(self.core.data.mode)
            }
            #[getter]
            fn get_is_completed(&self) -> PyResult<bool> {
                Ok(self.core.data.is_completed)
            }
            #[getter]
            fn get_is_official(&self) -> PyResult<bool> {
                Ok(self.core.data.is_official)
            }
            #[getter]
            fn get_is_fair(&self) -> PyResult<bool> {
                Ok(self.core.data.is_fair)
            }
            #[getter]
            fn get_mine_num(&self) -> PyResult<usize> {
                Ok(self.core.data.mine_num)
            }
            #[getter]
            fn get_player_identifier(&self) -> PyResult<String> {
                Ok(self.core.data.player_identifier.clone())
            }
            #[getter]
            fn get_race_identifier(&self) -> PyResult<String> {
                Ok(self.core.data.race_identifier.clone())
            }
            #[getter]
            fn get_uniqueness_identifier(&self) -> PyResult<String> {
                Ok(self.core.data.uniqueness_identifier.clone())
            }
            #[getter]
            fn get_country(&self) -> PyResult<String> {
                Ok(self.core.data.country.clone())
            }
            #[getter]
            fn get_bbbv(&self) -> PyResult<usize> {
                Ok(self.core.data.static_params.bbbv)
            }
            #[getter]
            fn get_start_time(&self) -> PyResult<u64> {
                Ok(self.core.data.start_time)
            }
            #[getter]
            fn get_end_time(&self) -> PyResult<u64> {
                Ok(self.core.data.end_time)
            }
            #[getter]
            fn get_op(&self) -> PyResult<usize> {
                Ok(self.core.data.static_params.op)
            }
            #[getter]
            fn get_isl(&self) -> PyResult<usize> {
                Ok(self.core.data.static_params.isl)
            }
            #[getter]
            fn get_hizi(&self) -> PyResult<usize> {
                Ok(self.core.data.static_params.hizi)
            }
            #[getter]
            fn get_cell0(&self) -> PyResult<usize> {
                Ok(self.core.data.static_params.cell0)
            }
            #[getter]
            fn get_cell1(&self) -> PyResult<usize> {
                Ok(self.core.data.static_params.cell1)
            }
            #[getter]
            fn get_cell2(&self) -> PyResult<usize> {
                Ok(self.core.data.static_params.cell2)
            }
            #[getter]
            fn get_cell3(&self) -> PyResult<usize> {
                Ok(self.core.data.static_params.cell3)
            }
            #[getter]
            fn get_cell4(&self) -> PyResult<usize> {
                Ok(self.core.data.static_params.cell4)
            }
            #[getter]
            fn get_cell5(&self) -> PyResult<usize> {
                Ok(self.core.data.static_params.cell5)
            }
            #[getter]
            fn get_cell6(&self) -> PyResult<usize> {
                Ok(self.core.data.static_params.cell6)
            }
            #[getter]
            fn get_cell7(&self) -> PyResult<usize> {
                Ok(self.core.data.static_params.cell7)
            }
            #[getter]
            fn get_cell8(&self) -> PyResult<usize> {
                Ok(self.core.data.static_params.cell8)
            }
            #[getter]
            fn get_rtime(&self) -> PyResult<f64> {
                Ok(self.core.data.get_rtime().unwrap())
            }
            #[getter]
            fn get_rtime_ms(&self) -> PyResult<u32> {
                Ok(self.core.data.get_rtime_ms().unwrap())
            }
            #[getter]
            fn get_etime(&self) -> PyResult<f64> {
                Ok(self.core.data.get_etime().unwrap())
            }
            #[getter]
            pub fn get_video_start_time(&self) -> PyResult<f64> {
                Ok(self.core.data.get_video_start_time().unwrap())
            }
            #[getter]
            pub fn get_video_end_time(&self) -> PyResult<f64> {
                Ok(self.core.data.get_video_end_time().unwrap())
            }
            #[getter]
            fn get_bbbv_s(&self) -> PyResult<f64> {
                Ok(self.core.data.get_bbbv_s().unwrap())
            }
            #[getter]
            fn get_stnb(&self) -> PyResult<f64> {
                Ok(self.core.data.get_stnb().unwrap())
            }
            #[getter]
            fn get_rqp(&self) -> PyResult<f64> {
                Ok(self.core.data.get_rqp().unwrap())
            }
            #[getter]
            fn get_left(&self) -> PyResult<usize> {
                Ok(self.core.data.get_left())
            }
            #[getter]
            fn get_right(&self) -> PyResult<usize> {
                Ok(self.core.data.get_right())
            }
            #[getter]
            fn get_double(&self) -> PyResult<usize> {
                Ok(self.core.data.get_double())
            }
            #[getter]
            fn get_cl(&self) -> PyResult<usize> {
                Ok(self.core.data.get_cl())
            }
            #[getter]
            fn get_flag(&self) -> PyResult<usize> {
                Ok(self.core.data.get_flag())
            }
            #[getter]
            fn get_bbbv_solved(&self) -> PyResult<usize> {
                Ok(self.core.data.get_bbbv_solved().unwrap())
            }
            #[getter]
            fn get_lce(&self) -> PyResult<usize> {
                Ok(self.core.data.get_lce().unwrap())
            }
            #[getter]
            fn get_rce(&self) -> PyResult<usize> {
                Ok(self.core.data.get_rce().unwrap())
            }
            #[getter]
            fn get_dce(&self) -> PyResult<usize> {
                Ok(self.core.data.get_dce().unwrap())
            }
            #[getter]
            fn get_ce(&self) -> PyResult<usize> {
                Ok(self.core.data.get_ce().unwrap())
            }
            #[getter]
            fn get_left_s(&self) -> PyResult<f64> {
                Ok(self.core.data.get_left_s())
            }
            #[getter]
            fn get_right_s(&self) -> PyResult<f64> {
                Ok(self.core.data.get_right_s())
            }
            #[getter]
            fn get_double_s(&self) -> PyResult<f64> {
                Ok(self.core.data.get_double_s())
            }
            #[getter]
            fn get_cl_s(&self) -> PyResult<f64> {
                Ok(self.core.data.get_cl_s())
            }
            #[getter]
            fn get_flag_s(&self) -> PyResult<f64> {
                Ok(self.core.data.get_flag_s())
            }
            #[getter]
            fn get_path(&self) -> PyResult<f64> {
                Ok(self.core.data.get_path())
            }
            #[getter]
            fn get_ce_s(&self) -> PyResult<f64> {
                Ok(self.core.data.get_ce_s().unwrap())
            }
            #[getter]
            fn get_ioe(&self) -> PyResult<f64> {
                Ok(self.core.data.get_ioe().unwrap())
            }
            #[getter]
            fn get_thrp(&self) -> PyResult<f64> {
                Ok(self.core.data.get_thrp().unwrap())
            }
            #[getter]
            fn get_corr(&self) -> PyResult<f64> {
                Ok(self.core.data.get_corr().unwrap())
            }
            #[getter]
            fn get_pluck(&self) -> PyResult<f64> {
                Ok(self.core.data.get_pluck().unwrap())
            }
            #[getter]
            fn get_events(&self) -> PyResult<Vec<PyVideoActionStateRecorder>> {
                Ok(self
                    .core
                    .data
                    .video_action_state_recorder
                    .iter()
                    .map(|x| PyVideoActionStateRecorder { core: x.clone() })
                    .collect())
            }
            // #[getter]
            // fn get_game_board_stream(&self) -> PyResult<Vec<PyGameBoard>> {
            //     Ok(self
            //         .core
            //         .data
            //         .game_board_stream
            //         .iter()
            //         .map(|x| PyGameBoard { core: x.clone() })
            //         .collect())
            // }
            #[getter]
            pub fn get_current_event_id(&self) -> PyResult<usize> {
                Ok(self.core.data.current_event_id)
            }
            #[setter]
            pub fn set_current_event_id(&mut self, id: usize) {
                self.core.data.current_event_id = id
            }
            #[getter]
            pub fn get_game_board(&self) -> PyResult<Vec<Vec<i32>>> {
                Ok(self.core.data.get_game_board())
            }
            #[getter]
            pub fn get_game_board_poss(&mut self) -> PyResult<Vec<Vec<f64>>> {
                Ok(self.core.data.get_game_board_poss())
            }
            #[getter]
            pub fn get_mouse_state(&self) -> PyResult<usize> {
                match self.core.data.video_action_state_recorder[self.core.data.current_event_id]
                    .mouse_state
                {
                    MouseState::UpUp => Ok(1),
                    MouseState::UpDown => Ok(2),
                    MouseState::UpDownNotFlag => Ok(3),
                    MouseState::DownUp => Ok(4),
                    MouseState::Chording => Ok(5),
                    MouseState::ChordingNotFlag => Ok(6),
                    MouseState::DownUpAfterChording => Ok(7),
                    MouseState::Undefined => Ok(8),
                }
            }
            /// 局面状态（录像播放器的局面状态始终等于6，没有ready、win、fail的概念）
            #[getter]
            pub fn get_game_board_state(&self) -> PyResult<usize> {
                match self.core.data.game_board_state {
                    GameBoardState::Ready => Ok(1),
                    GameBoardState::Playing => Ok(2),
                    GameBoardState::Win => Ok(3),
                    GameBoardState::Loss => Ok(4),
                    GameBoardState::PreFlaging => Ok(5),
                    GameBoardState::Display => Ok(6),
                }
            }
            #[getter]
            pub fn get_x_y(&self) -> PyResult<(u16, u16)> {
                Ok(self.core.data.get_x_y().unwrap())
            }
            #[getter]
            pub fn get_checksum(&self) -> PyResult<Vec<u8>> {
                Ok(self.core.data.get_checksum().unwrap())
            }
            #[getter]
            pub fn get_pix_size(&self) -> PyResult<u8> {
                Ok(self.core.data.get_pix_size().unwrap())
            }
            #[setter]
            pub fn set_current_time(&mut self, time: f64) {
                self.core.data.set_current_time(time);
            }
            pub fn is_valid(&self) -> PyResult<u8> {
                Ok(self.core.data.is_valid())
            }
            #[setter]
            pub fn set_video_playing_pix_size(&mut self, pix_size: u8) {
                self.core.data.set_video_playing_pix_size(pix_size);
            }
            // 读取录像文件后，有可能要转存。此时，例如rmv，录像中的国家信息是用户手动输入的
            // 需要在外部传入。因为所有国家的文本信息有39k，放在工具箱里不合适
            #[setter]
            pub fn set_country(&mut self, country: String) {
                self.core.data.set_country(country).unwrap();
            }
        }
    };
}
generate_video!(AvfVideo);
generate_video!(EvfVideo);
generate_video!(MvfVideo);
generate_video!(RmvVideo);
