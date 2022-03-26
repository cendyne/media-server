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

## Next things to do

* Refactor lookup handler code
* Custom NamedFile response with custom headers
* Send headers on objects with custom named file response
* Store headers on objects
* Send custom headers on objects
* Custom error type and error response
* Better virtual object choice by resolution
* Add file extension as content type parameter
* Add handler path to include width and height, rather than query parameter
* Use transaction around inserts with last insert id

