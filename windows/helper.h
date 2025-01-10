#ifndef HELPER_H
#define HELPER_H

#define CHECK_FAILED(hr) if (FAILED(hr)) { PRINT_LOG(hr);  goto done; }
#define PRINT_LOG(hr) std::cout << "Error at " << __FILE__ << ":" << static_cast<double>(__LINE__) << " => " << std::hex << hr << std::endl;

#endif // !HELPER_H
