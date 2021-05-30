# Changelog

Only versions published to the [registry](https://crates.io/crates/yayagram/versions) are documented.

<!--Order: Added, Changed, Fixed, Removed-->

## 0.5.0 (2021-05-30)

### Added

* Add the ability to temporarily darken all cells in the direction indicated using W, A, S, D or the arrow keys.

### Changed

* Change `M` key for the measurement tool to `X`.
* Amend measurement tool help text at the bottom.
* Make message shown when the window size is too small more helpful.

### Fixed

* Fix crash when pressing unknown key.
* Fix formatting of the time taken to solve the grid.

### Removed

* Remove the ability to save with `S`. Saving is now done only with `Enter`.
* Remove the ability to undo and redo with the left and right arrow keys respectively.

## 0.4.0 (2021-05-29)

### Added

* Added a measurement tool. Its usage is tracked by the undo redo buffer.

### Changed

* No longer clear loaded `.yaya` file for the period of the session.
* Improve the message for insufficient window size.

### Fixed

* Make the maximum grid size 99x99.

## 0.3.1 (2021-05-28)

### Fixed

* Fix crash when pressing space.

## 0.3.0 (2021-05-28)

### Added

* Allow passing both a width and height from the command line.
* Add a small internal basis for a measurement tool.

### Changed

* Allow non-squared grids.
* Don't crash when flushing the `.yaya` file failed.

## 0.2.5 (2021-05-24)

### Added

* `--help` and `-h` are now supported.

### Changed

* Default grid size is now 5x5 instead of 10x10.

### Fixed

* Fix dark cell color sometimes not being drawn.
* Fix `-` being detected as a number.

## 0.2.0 (2021-05-24)

### Added

This is a non-exhaustive listing.

* Solvable grids.
* Editor.
* Undo redo buffer.