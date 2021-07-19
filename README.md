![Java](https://badgen.net/badge/language/Java/green)
![Java](https://badgen.net/badge/Java/JDK-15/green)
[![GitHub license](https://badgen.net/github/license/maxwai/download-renamer-mover)](LICENSE)

# Download Watcher, renamer and mover Tool

The source code my personal Bot that Watches a specific Folder and tries to move a downloaded Video
to the correct Folder. If it cannot manage to move a Video it will ask the user over discord for a
mapping that will be saved.

## Getting Started

### Prerequisites

You will need Java Version 15 or later to make it work. It may work with lower Java versions, but it
was programmed using the Java 15 JDK.

**This Bot is supposed to be running on only one Discord Server at a Time.**

### Installing

Compile the source Code to a jar or execute it yourself however you want. You will need to create a
Config.xml file (A dummy file will be created by the program if not present, and you will be asked
to fill it)

### How to Use with a Jar

* Start the jar file in a terminal with the
  command `java -jar jar-file-name.jar <path to root folder>` <br>
  (do not just double click it to open it)
* All commands are only in Discord, command line commands are not necessary since the Bot is
  supposed to be run as a daemon and a service for easiness
* It is possible to stop the bot by sending `!stop` over Discord
* per default only `avi, mp4, mkv` are supported but others can be easily added
* The root folder specified is per default assumed to be build as follows:
  ```
  root folder
  |-- Download (Folder where the Videos are Downloaded to) (Can be changed in the Code)
  `-- Shared Video (Can be changed in the Code)
      |-- Anime (Folder where all the Anime is saved to) (Can be changed in the Code)
      |   |-- Anime 1
      |   |   |-- Staffel XY (Can be changed in the Code)
      |   |   |   |-- Anime 1 - sXYe01.mkv
      |   |   |   `-- ...
      |   |   `-- ...
      |   `-- ...
      `-- Serien (Folder where all other Series are saved to) (Can be changed in the Code)
          `-- ... (Same as for Anime)
  ```