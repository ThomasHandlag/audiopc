#include <string>
#include <mfapi.h>
#include <mfidl.h>
#include <map>
#include <flutter/encodable_value.h>

namespace audiopc {

	inline void wstringToString(const std::wstring& wstr, std::string& str) {
		int sizeNeeded = WideCharToMultiByte(CP_UTF8, 0, wstr.c_str(), -1, nullptr, 0, nullptr, nullptr);
		str = std::string(sizeNeeded, '\0');
		WideCharToMultiByte(CP_UTF8, 0, wstr.c_str(), -1, &str[0], sizeNeeded, nullptr, nullptr);
	}

	class AudioMetaData {
	
	public:
		AudioMetaData(const std::wstring path);
		~AudioMetaData();

		std::map<std::string, flutter::EncodableValue> metaData;
	private:
		HRESULT CreateMediaSource(const std::wstring& path, IMFMediaSource **pSource);

		HRESULT RetrieveMetadata(IMFMediaSource* pSource);

		HRESULT SetMetadata(const PROPVARIANT& var, const std::wstring& name);

		void SetData(const std::wstring& name, const std::wstring& value);
	};
}