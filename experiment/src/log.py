from datetime import datetime
from enum import IntEnum
import inspect

class LogLevel(IntEnum):
    NoLog = 0
    Error = 1
    Warn = 2
    Info = 3
    Debug = 4
    Trace = 5

LOG_NAMES = {
        LogLevel.Error: "ERROR",
        LogLevel.Warn: "WARN",
        LogLevel.Info: "INFO",
        LogLevel.Debug: "DEBUG",
        LogLevel.Trace: "TRACE"
}

__LOG_LEVEL = LogLevel.Error

def set_log_level(log_level: LogLevel):
    global __LOG_LEVEL
    __LOG_LEVEL = log_level

def error(to_print):
    write(LogLevel.Error, to_print, 2)

def warn(to_print):
    write(LogLevel.Warn, to_print, 2)

def info(to_print):
    write(LogLevel.Info, to_print, 2)

def debug(to_print):
    write(LogLevel.Debug, to_print, 2)

def trace(to_print):
    write(LogLevel.Trace, to_print, 2)

def write(log_level: LogLevel, to_print, frame_level = 1):
    if __LOG_LEVEL < log_level:
        return
    
    time = datetime.now()

    caller_frame = inspect.stack()[frame_level]
    module_name = inspect.getmodulename(caller_frame[1])
    lineno = caller_frame[2]

    print('[{} {} {}:{}] {}'.format(time, LOG_NAMES[log_level], module_name, lineno, to_print))

