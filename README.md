<img align="left" width="120" height="120" src="/logo.png">

<div id="user-content-toc">
  <ul style="list-style: none;">
    <summary>
      <h1>CrossCode 27</h1>
    </summary>
  </ul>
</div>

---

[![Build CrossCode](https://github.com/NovaDev404/CrossCode27/actions/workflows/build.yml/badge.svg)](https://github.com/nab138/CrossCode/actions/workflows/build.yml)

iOS Swift development IDE for Windows/Linux. Create, build, and test apps without owning a Mac.

Supports Swift 6.2 and 6.4, Xcode 26 and 27, and the Swift Package Manager.

### Demo

https://github.com/user-attachments/assets/9cbe2b71-d765-46c6-aa25-ef16b539deec

## Installation

CrossCode is currently in alpha. Expect bugs!

Download the latest build for your platform from [releases](https://github.com/NovaDev404/CrossCode27/releases/latest).

Check out the [Getting Started](https://github.com/nab138/CrossCode/wiki#getting-started) section of the [wiki](https://github.com/nab138/CrossCode/wiki). Also, see [Troubleshooting](https://github.com/nab138/CrossCode/wiki/Troubleshooting) and [FAQ](https://github.com/nab138/CrossCode/wiki/FAQ)

## Features

- Generate a Darwin SDK for Linux from a user-provided copy of Xcode 26 or 27 to build the apps
- Build apps using swift package manager
- Log in with your Apple ID to sign apps
- Install apps on device
- Create projects from templates
- Code editing including error reporting, autocomplete, go to definition, and other language features
- Light/dark mode and other customizations
- View and manage certificates, app IDs, and more
- View the syslog or the stdout (console) of your device/app
- Much more (and more to come!)

## Future plans

The app is currently functional but does not have all the features it should. You can see a tentative plan for the future [on trello](https://trello.com/b/QYQFfOvm/ycode)

Please note that I am one person, so development may be slow. If you want to help, PRs welcome!

## How it works

- A Darwin SDK is generated from a user-provided copy of Xcode 26 or 27 (extracted with [unxip-rs](https://github.com/nab138/unxip-rs)) and Darwin tools from [darwin-tools-linux-llvm](https://github.com/xtool-org/darwin-tools-linux-llvm)
- Swift uses the darwin SDK to build an executable which is packaged into an .app bundle.
- The code to sign and install apps onto a device has been removed from CrossCode's source and moved to a standalone package, [isideload](https://github.com/nab138/isideload). It was built on a lot of other libraries, so check out its README for more info.

## Credits

- [NovaDev404](https://github.com/NovaDev404/) for adding Xcode 27 support
- [nab138](https://github.com/nab138/) for creating CrossCode
- [idevice](https://github.com/jkcoxson/idevice) is used to communicate with iOS devices.
- [xtool](https://xtool.sh) has been used as a reference for the implementation of the darwin SDK generation.
- [Sideloader](https://github.com/Dadoum/Sideloader) has been heavily used as a reference for the implementation of the Apple Developer APIs and sideloading process.
- [GNU cpio](https://www.gnu.org/software/cpio/) 2.14 is included under GPLv3 (see licenses/GPL-3.0.txt), with its copyright holders. See [Source code](https://ftp.gnu.org/gnu/cpio/cpio-2.14.tar.gz). It is used to help with XIP extraction.

### AI Usage

- NovaDev404 used AI to help add Xcode 27 support
- Nab138 used AI to help port small sections of code from [Sideloader](https://github.com/Dadoum/Sideloader) because not familiar with dlang syntax
