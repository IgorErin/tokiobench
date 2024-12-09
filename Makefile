.PHONY: bench
bench:
	python3 .benchpy/bench.py

.PHONY: clean
clean:
	cargo clean

	cd tokio; cargo clean
	cd tokio-metrics; cargo clean

.PHONY: plot
plot:
	python3 .benchpy/plot.py

.PHONY: metr
metr:
	python3 .benchpy/metric.py

.PHONY: check
check:
	cargo check
	cargo bench --no-run -F check
