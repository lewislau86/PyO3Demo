import rust_core
import time


def count_primes(limit):
	count = 0

	for number in range(2, limit + 1):
		is_prime = True
		divisor = 2

		while divisor * divisor <= number:
			if number % divisor == 0:
				is_prime = False
				break
			divisor += 1

		if is_prime:
			count += 1

	return count


def benchmark(name, function, limit):
	start = time.perf_counter()
	result = function(limit)
	elapsed = time.perf_counter() - start
	print(f"{name}: {result} primes in {elapsed:.4f} seconds")
	return result, elapsed


if __name__ == "__main__":
	limit = 500_000

	if rust_core.apply_twice(lambda value: value + 3, 10) != 16:
		raise RuntimeError("Rust callback invocation returned an unexpected result")
	if rust_core.call_upper("hello") != "HELLO":
		raise RuntimeError("Rust Python method invocation returned an unexpected result")

	python_result, python_time = benchmark("Python", count_primes, limit)
	rust_result, rust_time = benchmark("Rust  ", rust_core.count_primes, limit)
	rust_detached_result = rust_core.count_primes_without_gil(limit)

	if python_result != rust_result or rust_result != rust_detached_result:
		raise RuntimeError("Python and Rust returned different results")

	print(f"Rust speedup: {python_time / rust_time:.2f}x")
