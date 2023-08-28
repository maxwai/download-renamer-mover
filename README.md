![Rust: 1.60+](https://img.shields.io/badge/rust-1.71.1+-93450a)
[![GitHub license](https://badgen.net/github/license/maxwai/download-renamer-mover)](LICENSE)

# Download Watcher, renamer and mover Tool

The source code my personal Bot that Watches a specific Folder and tries to move a downloaded Video
to the correct Folder. If it cannot manage to move a Video it will ask the user over discord for a
mapping that will be saved.

## Getting Started

### Prerequisites

You will need version 1.71.1 of the rust compiler to make it work. It may work with lower versions, but it was
programmed using the 1.71.1 compiler.

**This Bot is supposed to be running on only one Discord Server at a Time.**

### Installing

Compile the source Code via cargo. You will need to create a Config.xml file (A dummy file will be created by the
program if not present, and you will be asked to fill it)

### How to Use

* Start the compiled binary
  * If you want to specify the path to the root folder, add the path as a programm argument
* All commands are only in Discord, command line commands are not necessary since the Bot is
  supposed to be run as a daemon / service
* It is possible to stop the bot by sending `!stop` over Discord
* per default only `avi, mp4, mkv` are supported but others can be easily added
* The root folder specified is per default assumed to be build as follows:
  ```
  root folder
  |-- Download (Folder where the Videos are Downloaded to) (Name can be changed in the Code)
  `-- Shared Video (Name can be changed in the Code)
      |-- Anime (Folder where all the Anime is saved to) (Name can be changed in the Code)
      |   |-- Anime 1
      |   |   |-- Staffel XY (Name can be changed in the Code)
      |   |   |   |-- Anime 1 - sXYeZZ.mkv
      |   |   |   `-- ...
      |   |   `-- ...
      |   `-- ...
      `-- Serien (Folder where all other Series are saved to) (Name can be changed in the Code)
          `-- ... (Same as for Anime)
  ```