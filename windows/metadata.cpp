#include "metadata.h"
#include <mfapi.h>
#include <mfidl.h>
#include <Shlwapi.h>
#include <Mferror.h>
#include <iostream>
#include <propkey.h>

#pragma comment(lib, "Mf.lib")
#pragma comment(lib, "propsys.lib")

namespace audiopc {

	using std::wstring, std::cout, std::endl;

	AudioMetaData::AudioMetaData(const wstring path) {
		HRESULT hr = S_OK;
		IMFMediaSource* m_pSource = NULL;
		metaData = std::map<std::string, flutter::EncodableValue>();
		hr = MFStartup(MF_VERSION);
		if (FAILED(hr)) {
			goto done;
		}
		hr = CreateMediaSource(path.c_str(), &m_pSource);
		if (FAILED(hr)) {
			goto done;
		}

		hr = RetrieveMetadata(m_pSource);
		if (FAILED(hr)) {
			goto done;
		}

	done:
		if (m_pSource) {
			m_pSource->Release();
		}
		return;
	}

	AudioMetaData::~AudioMetaData() {
		metaData.clear();
		MFShutdown();
	}

	HRESULT AudioMetaData::CreateMediaSource(const std::wstring& filePath, IMFMediaSource** ppSource) {
		if (filePath.empty() || !ppSource) {
			return E_INVALIDARG;
		}

		HRESULT hr = S_OK;
		IMFSourceResolver* pResolver = nullptr;

		try {
			// Create source resolver
			hr = MFCreateSourceResolver(&pResolver);
			if (FAILED(hr)) {
				std::wcerr << L"Failed to create source resolver: "
					<< std::hex << hr << std::endl;
				goto done;
			}

			// Determine source type
			MF_OBJECT_TYPE ObjectType = MF_OBJECT_INVALID;

			// Create media source
			hr = pResolver->CreateObjectFromURL(
				filePath.c_str(),
				MF_RESOLUTION_MEDIASOURCE,
				nullptr,
				&ObjectType,
				reinterpret_cast<IUnknown**>(ppSource)
			);

			if (FAILED(hr)) {
				goto done;
			}

			return hr;
		}
		catch (const std::exception& e) {
			std::wcerr << L"Exception during media source creation: "
				<< e.what() << std::endl;
			return E_UNEXPECTED;
		}
	done:
		if (pResolver) pResolver->Release();
		return hr;
	}

	HRESULT AudioMetaData::RetrieveMetadata(IMFMediaSource* pSource) {
		if (!pSource) {
			std::wcerr << L"Invalid media source" << std::endl;
			return E_INVALIDARG;
		}

		HRESULT hr = S_OK;
		IMFMetadataProvider* pProvider = nullptr;
		IMFMetadata* pMetadata = nullptr;
		PROPVARIANT var;
		PropVariantInit(&var);

		try {
			// Query for metadata provider
			hr = pSource->QueryInterface(IID_PPV_ARGS(&pProvider));
			if (FAILED(hr)) {
				std::wcerr << L"Failed to get metadata provider: "
					<< std::hex << hr << std::endl;
				goto done;
			}

			// Get metadata
			hr = pProvider->GetMFMetadata(nullptr, 0, 0, &pMetadata);
			if (FAILED(hr)) {
				goto done;
			}

			// Get all property names
			hr = pMetadata->GetAllPropertyNames(&var);
			if (FAILED(hr)) {
				goto done;
			}

			// Check if we have a vector of property names
			if (var.vt == (VT_VECTOR | VT_LPWSTR)) {
				for (ULONG i = 0; i < var.calpwstr.cElems; ++i) {
					PROPVARIANT varValue;
					PropVariantInit(&varValue);

					hr = pMetadata->GetProperty(var.calpwstr.pElems[i], &varValue);
					if (SUCCEEDED(hr)) {
						SetMetadata(varValue, var.calpwstr.pElems[i]);
						PropVariantClear(&varValue);
					}
				}
			}
			else {
				return E_FAIL;
			}

			return S_OK;
		}
		catch (const std::exception& e) {
			std::wcerr << L"Exception during metadata retrieval: "
				<< e.what() << std::endl;
			return E_UNEXPECTED;
		}
	done:
		// Cleanup
		PropVariantClear(&var);
		if (pProvider) pProvider->Release();
		if (pMetadata) pMetadata->Release();
		return hr;
	}

	HRESULT AudioMetaData::SetMetadata(const PROPVARIANT& value, const wstring& pName) {
		if (pName.empty()) {
			return E_INVALIDARG;
		}
		switch (value.vt) {
		case VT_EMPTY: {
			break;
		}

		case VT_UI4: {
			cout << value.ulVal << endl;
			break;
		}
		case VT_UI8: {
			cout << value.uhVal.QuadPart << endl;
			break;
		}
		case VT_R8: {
			cout << value.dblVal << endl;
			break;
		}
		case VT_BSTR: {
			std::wstring wValue(value.bstrVal);
			SetData(pName, wValue);
			break;
		}
		case VT_I4:
			std::wcout << value.lVal << std::endl;
			break;
		case VT_I8:
			std::wcout << value.hVal.QuadPart << std::endl;
			break;
		case VT_VECTOR | VT_LPWSTR: {
			for (ULONG i = 0; i < value.calpwstr.cElems; ++i) {
				std::wcout << value.calpwstr.pElems[i] << L", ";
			}
			std::wcout << std::endl;
			break;
		}
		case VT_LPWSTR: {
			std::wstring wValue(value.pwszVal);
			SetData(pName, wValue);
			break;
		}
		case VT_BLOB: {
			if (pName.compare(L"WM/Picture") == 0) {
				BYTE* artworkData = new BYTE[value.blob.cbSize];
				memcpy(artworkData, value.blob.pBlobData, value.blob.cbSize);

				std::vector<uint8_t> artwork(value.blob.cbSize);
				for (UINT i = 0; i < value.blob.cbSize; i++) {
					artwork[i] = artworkData[i];
				}
				metaData.insert({ "artwork", flutter::EncodableValue(artwork) });
			}
			break;
		}
		default: {
			std::wcout << L"Unsupported value type for property: " << pName << std::endl;
			break;
		}
		}

		return S_OK;
	}

	void AudioMetaData::SetData(const std::wstring& name, const std::wstring& value) {
		if (name.empty() || value.empty()) {
			return;
		}
		if (name.compare(L"Title") == 0) {
			std::string narrowStr;
			wstringToString(value, narrowStr);
			metaData.insert({ "title", flutter::EncodableValue(narrowStr) });
		}
		else if (name.compare(L"Author") == 0) {
			int sizeNeeded = WideCharToMultiByte(CP_UTF8, 0, value.c_str(), -1, nullptr, 0, nullptr, nullptr);
			std::string narrowStr(sizeNeeded, '\0');
			WideCharToMultiByte(CP_UTF8, 0, value.c_str(), -1, &narrowStr[0], sizeNeeded, nullptr, nullptr);
			metaData.insert({ "artist", flutter::EncodableValue(narrowStr) });
		}
		else if (name.compare(L"WM/Album") == 0) {
			int sizeNeeded = WideCharToMultiByte(CP_UTF8, 0, value.c_str(), -1, nullptr, 0, nullptr, nullptr);
			std::string narrowStr(sizeNeeded, '\0');
			WideCharToMultiByte(CP_UTF8, 0, value.c_str(), -1, &narrowStr[0], sizeNeeded, nullptr, nullptr);
			metaData.insert({ "album", flutter::EncodableValue(narrowStr) });
		}
		else if (name.compare(L"WM/CopyRight") == 0) {
			int sizeNeeded = WideCharToMultiByte(CP_UTF8, 0, value.c_str(), -1, nullptr, 0, nullptr, nullptr);
			std::string narrowStr(sizeNeeded, '\0');
			WideCharToMultiByte(CP_UTF8, 0, value.c_str(), -1, &narrowStr[0], sizeNeeded, nullptr, nullptr);
			metaData.insert({ "copyRight", flutter::EncodableValue(narrowStr) });
		}
		else if (name.compare(L"WM/Genre") == 0) {
			int sizeNeeded = WideCharToMultiByte(CP_UTF8, 0, value.c_str(), -1, nullptr, 0, nullptr, nullptr);
			std::string narrowStr(sizeNeeded, '\0');
			WideCharToMultiByte(CP_UTF8, 0, value.c_str(), -1, &narrowStr[0], sizeNeeded, nullptr, nullptr);
			metaData.insert({ "genre", flutter::EncodableValue(narrowStr) });
		}
		else if (name.compare(L"WM/AlbumTitle") == 0) {
			int sizeNeeded = WideCharToMultiByte(CP_UTF8, 0, value.c_str(), -1, nullptr, 0, nullptr, nullptr);
			std::string narrowStr(sizeNeeded, '\0');
			WideCharToMultiByte(CP_UTF8, 0, value.c_str(), -1, &narrowStr[0], sizeNeeded, nullptr, nullptr);
			metaData.insert({ "albumTitle", flutter::EncodableValue(narrowStr) });
		}
	}
}

