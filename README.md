# media-server
Low memory media storage and metadata tracker.
With "virtual objects", can select from multiple objects depending on requested content type, width, and height.
While primarily for images it can store other common file types too.

## Things done

1. Uploading Objects
2. Creating Virtual Objects
3. Asserting Virtual Object relationships
4. File lookup by object path
5. Virtual object path lookup
6. Refactor and improve content type to extension mapping
7. Extension to content type mapping
8. Refactor lookup handler code
9. Support multiple path lookup for virtual object
10. Parse content extension and encoding extension
11. Refactored content type and extension code
12. Refactored object, virtual object, find, and hash code

## Next things to do

* Extension and encoding extension to content type and content encoding
* Redo file extension as content type parameter
* Filter content type in database query

* Prioritized content type
* Custom NamedFile response with custom headers
* Send headers on objects with custom named file response
* Store headers on objects
* Send custom headers on objects
* Custom error type and error response
* Better virtual object choice by resolution
* Virtual Object info endpoint
* Virtual Object tags
* Virtual Object path prefixes
* Create virtual object for object if path is set
* No longer rely upon public path on object
* Add handler path to include width and height, rather than query parameter
* Use transaction around inserts with last insert id
* Determine content type and encoding by extension (e.g. `.js.gz => text/javascript gzip`)
    > https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Encoding
    > https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Accept-Encoding
* Add support for content encoding
* Add support for responding with content encoding depending on extension

