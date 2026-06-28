import Foundation
import CArchivalCore

public enum ArchivalDBError: Error, LocalizedError {
    case openFailed(String)
    case dbError(String)
    case notFound(String)
    case invalidInput(String)
    case ioError(String)
    case jsonError(String)
    case unknown

    public var errorDescription: String? {
        switch self {
        case .openFailed(let m):   return "DB open failed: \(m)"
        case .dbError(let m):     return "DB error: \(m)"
        case .notFound(let m):    return "Not found: \(m)"
        case .invalidInput(let m): return "Invalid input: \(m)"
        case .ioError(let m):     return "IO error: \(m)"
        case .jsonError(let m):   return "JSON error: \(m)"
        case .unknown:            return "Unknown Archival error"
        }
    }

    static func fromResult(_ result: ArchivalResult, handle: OpaquePointer?) -> ArchivalDBError {
        let msg: String
        if let ptr = archival_last_error() {
            msg = String(cString: ptr)
        } else {
            msg = "no detail"
        }
        switch result {
        case Ok:             return .unknown
        case ErrDb:          return .dbError(msg)
        case ErrNotFound:    return .notFound(msg)
        case ErrInvalidInput: return .invalidInput(msg)
        case ErrIo:          return .ioError(msg)
        case ErrJson:        return .jsonError(msg)
        default:             return .unknown
        }
    }
}
