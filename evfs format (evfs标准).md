## **evfs: An Open-Source Multi-Replay Format for Minesweeper**

* The full specification and its revision history are available at: [https://github.com/eee555/ms_toollib/blob/main/evfs%E6%A0%87%E5%87%86.md](https://github.com/eee555/ms_toollib/blob/main/evfs%E6%A0%87%E5%87%86.md)
* In the event of any conflict between language versions, the **Chinese version prevails**.

### **1.1 Purpose and Goals**

**evfs** (eee555’s Minesweeper Video Format Set) is a file format designed to store **multiple Minesweeper replays**, including all wins, losses, and restarts—but **excluding mouse actions between games**.

Evfs is not simply a container that bundles multiple replay files. It is intended to **record a sequence of consecutively generated Minesweeper replays over a period of time**, while providing **checksums** and addressing core requirements such as compatibility and interactivity. It is also helpful for computing statistics such as win rate and number of games played.

### **1.2 Terminology**

**Unit**:
An evfs file consists of several *units*, each containing one replay file along with its associated validation data.

### **1.3 Scope of Use**

evfs is designed for the following scenarios:

* Any Minesweeper software, replay tool, analysis tool, or competition system that needs to analyze, record, or replay multiple replays generated continuously over a period of time.
* All game modes defined in the EVF standard, covering various mine counts and board sizes.
* Saving replays generated during a single application session; once the software is closed, new replays must be stored in a new evfs file.
* Only for storing replays of a **single user**, not multi-user replay collections.
* Currently limited to EVF-format replays, though technically capable of supporting other formats.

### **1.4 Interaction Model**

Supported interactions:

* Query and iterate through EVF replays inside an evfs file.
* Index and export EVF replays from an evfs file.
* Append new replays to the **end** of the file while the Minesweeper application remains open.
* Delete the **last** replay from the evfs file on disk.

Unsupported interactions:

* Merging multiple EVF files into one evfs.
* Merging multiple evfs files into one.
* Inserting replays into the middle of an evfs file, or appending replays to an evfs file that was closed and reopened.

### **Format Requirements**

* All replays within an evfs file must share identical **software name and version**, **EVF version**, **country**, **user identifier**, **competition identifier**, and **unique identifier**.
* Replay start and end timestamps must be **strictly increasing** and must not overlap.

---

## **v0.0 Format (Used by Metasweeper 3.2.1)**

Format description:

1. **1 byte (uint8): version number.**
   The version described in this document is `'\0'`.

2. **2 bytes (uint16): checksum length** for each evfs unit.

3. **EVF replay data (repeated structure):**

   * Null-terminated UTF-8 string: **EVF replay filename**
   * **4 bytes (uint32):** size of the full EVF data block (in bytes)
   * Variable-length block: **EVF replay data**
   * Variable-length block: **unit checksum**

