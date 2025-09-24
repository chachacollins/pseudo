clean:
	fd -e .c --full-path './examples' -x rm
	fd -e .s --full-path './examples' -x rm
	fd -t x --full-path './examples' -x rm
