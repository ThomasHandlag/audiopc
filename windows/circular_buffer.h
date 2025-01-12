#pragma once

#include <iostream>
#include <vector>
namespace audiopc {

	using std::cout, std::endl, std::vector;

	class CircularBuffer {
	private:
		size_t maxSize;
		vector<double> buffer;
		size_t csor = 0;
	public:
		CircularBuffer(size_t size)
			: maxSize(size) {
			buffer = vector<double>(size, 0);
		}

		void Write(const vector<double>& samples) {
			try {
				for (double sample : samples) {
					if (csor == maxSize) {
						csor = 0;
					}
					buffer[csor] = sample;
					if (csor < maxSize) {
						csor++;
					}
				}
			}
			catch (const std::exception& e) {
				cout << "Error at " << __FILE__ << ":" << static_cast<int>(__LINE__) << " - " << e.what() << endl;
			}
		}

		void Read(vector<double>& out) const {
			try {
				for (size_t i = 0; i < csor; i++) {
					out.push_back(buffer[i]);
				}
			}
			catch (const std::exception& e) {
				cout << "Error at " << __FILE__ << ":" << static_cast<int>(__LINE__) << " - " << e.what() << endl;
			}
		}
	};

} // namespace audiopc

