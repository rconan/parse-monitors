Here are the steps to build a CFD report:

     1. batch_force:
          * cargo r -r --bin batch_force --features plot

     2. group-force (optional):
          * cargo r -r --bin group-force --features plot

     3. opd_maps:
          * cargo r -r --bin opd_maps --features plot

     4. pressure_maps:
          * cargo r -r --bin pressure_maps --features plot

     5. dome-seeing:
          * cargo r -r --bin dome-seeing --features plot
	 
	 6. cfd_report:
		  * cargo run --release --bin cfd_report -- --full 
		  * cd report
		  * pdflatex gmto.cfd2025.tex
		  * pdflatex gmto.cfd2025.tex
		  * pdflatex gmto.cfd2025.tex
