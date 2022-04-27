Here are the steps to build a CFD report:

     1. batch_force:
          * cargo build --release --bin batch_force --features plot
		  * sudo -E ./../target/release/batch_force --all

     2. group-force:
          * cargo build --release --bin group-force --features plot
		  * sudo -E ./../target/release/group-force

     3. opd_maps:
          * cargo build --release --bin opd_maps --features plot
		  * sudo -E ./../target/release/opd_maps

     4. pressure_maps:
          * cargo build --release --bin pressure*maps --features plot
		  * sudo -E ./../target/release/pressure*maps

     5. dome-seeing:
          * cargo build --release --bin dome-seeing --features plot
		  * sudo -E ./../target/release/dome-seeing
	 
	 6. cfd_report:
		  * cargo run --release --bin cfd_report -- --full 
		  * cd report
		  * pdflatex gmto.cfd2021.tex
		  * pdflatex gmto.cfd2021.tex
