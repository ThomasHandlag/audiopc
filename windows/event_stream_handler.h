#pragma once
#include <flutter/encodable_value.h>
#include <flutter/event_channel.h>
#include <map>
#include <iostream>

namespace audiopc {
	using flutter::EncodableValue, flutter::StreamHandler, std::unique_ptr, flutter::StreamHandlerError, std::map;
	using flutter::EventSink, std::string;
	class EventStreamHandler : StreamHandler<EncodableValue> {
	public:
		EventStreamHandler() : _sink(nullptr){}

		void emitEvent(const map<string, EncodableValue> value) const {
			if (_sink) {
				_sink->Success(EncodableValue(value));
			}
		}

	protected:
		unique_ptr<StreamHandlerError<EncodableValue>> 
			OnListenInternal(
				const EncodableValue* arguments, std::unique_ptr<flutter::EventSink<EncodableValue>>&& events
			) override
		{
			_sink = std::move(events);

			return nullptr;
		}

		std::unique_ptr<flutter::StreamHandlerError<flutter::EncodableValue>> OnCancelInternal(
			const flutter::EncodableValue* arguments) override {
			_sink = nullptr;
			return nullptr;
		}

		unique_ptr<EventSink<EncodableValue>> _sink;

	};
};