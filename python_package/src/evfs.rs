use crate::videos::EvfVideo;
use ms_toollib_original::*;
use pyo3::prelude::*;
use pyo3::types::{PyInt, PyList, PySlice};

#[pyclass(name = "EvfsCell", unsendable)]
pub struct PyEvfsCell {
    pub core: EvfsCell,
}

#[pymethods]
impl PyEvfsCell {
    #[getter]
    pub fn get_evf_video(&self) -> EvfVideo {
        EvfVideo {
            core: self.core.evf_video.clone(),
        }
    }
    #[getter]
    pub fn get_checksum(&self) -> PyResult<Vec<u8>> {
        Ok(self.core.checksum.clone())
    }
}

#[pyclass(name = "Evfs", unsendable)]
pub struct PyEvfs {
    pub core: Evfs,
}

#[pymethods]
impl PyEvfs {
    #[new]
    #[pyo3(signature = (file_name="", raw_data=vec![]))]
    pub fn new(file_name: &str, raw_data: Vec<u8>) -> Self {
        if raw_data.is_empty() {
            if file_name.is_empty() {
                PyEvfs { core: Evfs::new() }
            } else {
                PyEvfs {
                    core: Evfs::new_with_file(file_name),
                }
            }
        } else {
            PyEvfs {
                core: Evfs::new_with_data(raw_data),
            }
        }
    }
    pub fn push(&mut self, data: Vec<u8>, file_name: &str, checksum: Vec<u8>) {
        self.core.push(data, file_name, checksum);
    }
    pub fn pop(&mut self) {
        self.core.pop();
    }
    pub fn len(&self) -> usize {
        self.core.len()
    }
    pub fn is_empty(&self) -> bool {
        self.core.is_empty()
    }
    pub fn clear(&mut self) {
        self.core.clear();
    }
    /// 初步验证evfs文件的有效性。适用于网页前端，并不严格。
    pub fn is_valid(&mut self) -> bool {
        self.core.is_valid()
    }
    #[getter]
    pub fn get_file_name(&self) -> &str {
        &self.core.file_name
    }
    #[getter]
    pub fn get_software(&self) -> &str {
        &self.core.get_software()
    }
    #[getter]
    pub fn get_evf_version(&self) -> u8 {
        self.core.get_evf_version()
    }
    #[getter]
    pub fn get_start_time(&self) -> u64 {
        self.core.get_start_time()
    }
    #[getter]
    pub fn get_end_time(&self) -> u64 {
        self.core.get_end_time()
    }

    /// 生成evfs_v0文件的二进制数据
    pub fn generate_evfs_v0_raw_data(&mut self) {
        self.core.generate_evfs_v0_raw_data();
    }
    pub fn parse(&mut self) {
        self.core.parse().unwrap();
    }
    pub fn analyse(&mut self) {
        self.core.analyse().unwrap();
    }
    pub fn analyse_for_features(&mut self, controller: Vec<String>) {
        let controller_slice: &Vec<&str> = &controller.iter().map(|s| s.as_str()).collect::<Vec<_>>();
        self.core.analyse_for_features(controller_slice).unwrap();
    }
    pub fn save_evf_files(&self, dir: &str) {
        self.core.save_evf_files(dir);
    }
    pub fn save_evfs_file(&self, file_name: &str) -> PyResult<String> {
        let output_file_name = self.core.save_evfs_file(file_name);
        Ok(output_file_name)
    }
    pub fn __getitem__(&self, py: Python<'_>, key: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        // 先尝试当作整数索引（支持负索引）
        if let Ok(index_obj) = key.cast::<PyInt>() {
            let idx: isize = index_obj.extract()?;
            let adjusted_idx = if idx < 0 {
                (self.core.len() as isize + idx) as usize
            } else {
                idx as usize
            };
            if adjusted_idx < self.core.len() {
                let cell = PyEvfsCell {
                    core: self.core[adjusted_idx].clone(),
                };
                // 把 pyclass 包装成 PyObject 返回
                return Ok(Py::new(py, cell)?.into());
            } else {
                return Err(pyo3::exceptions::PyIndexError::new_err(
                    "Index out of range",
                ));
            }
        }

        // 再尝试当作 slice
        if let Ok(slice) = key.cast::<PySlice>() {
            // 使用 slice.indices 来把切片规范化为 (start, stop, step)
            let length = self.core.len() as isize;
            let indices = slice.indices(length)?;
            let mut result: Vec<Py<PyAny>> = Vec::with_capacity(indices.slicelength);

            let mut i = indices.start;
            let stop = indices.stop;
            let step = indices.step;

            if step > 0 {
                while i < stop {
                    let cell = PyEvfsCell {
                        core: self.core[i as usize].clone(),
                    };
                    result.push(Py::new(py, cell)?.into());
                    i += step;
                }
            } else {
                while i > stop {
                    let cell = PyEvfsCell {
                        core: self.core[i as usize].clone(),
                    };
                    result.push(Py::new(py, cell)?.into());
                    i += step; // step is negative here
                }
            }

            return Ok(PyList::new(py, result)?.into());
        }

        // 不是 int 也不是 slice，报类型错误
        Err(pyo3::exceptions::PyTypeError::new_err(
            "Invalid key type, expected int or slice",
        ))
    }
}
