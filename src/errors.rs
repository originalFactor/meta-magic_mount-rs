// Copyright (C) 2026 Tools-cx-app <localhost.hutao@gmail.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use thiserror::Error as ThisError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("cannot mount root symlink {path:?}!")]
    MountRootSymlink { path: String },
    #[error("dir {path:?} is declared as replaced but it is root!")]
    DirDeclared { path: String },
    #[error("cannot mount root file {path:?}!")]
    MountRootFile { path: String },
    #[error("{path:?} is not a regular directory")]
    RegularDirectory { path: String },
    #[error("Invalid module ID: '{module_id:?}'. Must match /^[a-zA-Z][a-zA-Z0-9._-]+$/")]
    InvalidModuleID { module_id: String },
    #[error("missing required --payload argument")]
    MissingArgment,
    #[error("hex payload must contain an even number of characters")]
    PayloadContain,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Glob(#[from] glob::PatternError),
    #[error(transparent)]
    Pproperties(#[from] java_properties::PropertiesError),
    #[error(transparent)]
    AnyHow(#[from] anyhow::Error),
    #[error(transparent)]
    SerJson(#[from] serde_json::Error),
    #[error(transparent)]
    Rustix(#[from] rustix::io::Errno),
    #[error(transparent)]
    Regex(#[from] regex_lite::Error),
}
