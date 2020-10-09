Implementation of the safe data storage in RUST PoC project

- Objectives
	* Demonstrate ability to store and manipulate data while having at least the same abilities as provided by C
	* Demonstrate C & RUST interoperability by being able to link together and call C->RUST->C
	* Emulate (although simplified) real NSM subsystem
	* Demonstrate built-in unit testing

- Description
	* RUST library which in turn consists of:
		- Emulated storage of a protocol peers
			- Each element has its routing table whose elements are also present in the global routing table
		- Global routing table whose entries
			- are referred by the elements of peers storage (see above)
			- Have list of peers it is referred by
		- Patricia tree which stores prefixes
			- Insert, lookup and remove API
		- Emulated FTN & ILM API
			- FTN entries can be insert, looked up and removed
			- ILM entries can be insert, looked up and removed
			- XC and NHLFE entries are created implicitly as needed (upon FTN/ILM entries creation)
			- NH table consists of next hops. If the next hop for FTN/ILM is not found in NH table, that entry is considered dependent
			- Some code which handles dependent FTN/ILM entries handling (although it is not a complete implementation)
			- Several methods to improve code quality are demonstrated. It includes: macros, generic (template) functions, traits (behavior inheritance)
			- Using destructor-like behavior is demonstrated (via Drop trait)

- How to build and run unittests
	* To build, run ./build.sh from the project  root. It will build both RUST and C code and run unit tests, if the build was successful

- How to run
	* ./run.sh from the project root

		
