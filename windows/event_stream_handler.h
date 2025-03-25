#pragma once
#include <flutter/encodable_value.h>
#include <flutter/event_channel.h>
#include <map>
#include <iostream>
#include <any>

namespace audiopc {
	using flutter::EncodableValue, flutter::StreamHandler, std::unique_ptr, flutter::StreamHandlerError, std::map;
	using flutter::EventSink, std::string, std::shared_ptr, std::any;

	class EventStreamHandler : public StreamHandler<EncodableValue> {
	public:
		EventStreamHandler() {}
		static shared_ptr<EventSink<EncodableValue>> _sink;
		static void setSink(unique_ptr<flutter::EventSink<EncodableValue>>&& sink) {
			_sink = move(sink);
		}

		static void destroySink() {
			_sink = nullptr;
		}
	protected:
		unique_ptr<StreamHandlerError<EncodableValue>> 
			OnListenInternal(
				const EncodableValue* arguments, std::unique_ptr<flutter::EventSink<EncodableValue>>&& events
			) override
		{
			setSink(move(events));
			return nullptr;
		}

		std::unique_ptr<flutter::StreamHandlerError<flutter::EncodableValue>> OnCancelInternal(
			const flutter::EncodableValue* arguments) override {
			destroySink();
			return nullptr;
		}
	};
};