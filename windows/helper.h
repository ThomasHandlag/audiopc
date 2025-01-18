#ifndef HELPER_H
#define HELPER_H

using std::cout, std::endl, std::hex;

constexpr char* DB_RED = "\033[1;31m";
constexpr char* DB_GREEN = "\033[1;32m";
constexpr char* DB_YELLOW = "\033[1;33m";
constexpr char* DB_BLUE = "\033[1;34m";
constexpr char* DB_MAGENTA = "\033[1;35m";
constexpr char* DB_CYAN = "\033[1;36m";
constexpr char* DB_RESET = "\033[0m";
constexpr char* DB_BOLD = "\033[1m";

enum ERROR_TYPE {
	WARNING,
	FATAL
};

#define CHECK_FAILED(hr) if (FAILED(hr)) { goto done; }

#endif // !HELPER_H
