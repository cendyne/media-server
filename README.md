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
13. Redo file extension as content type parameter
14. Store content encoding (as input) onto record
15. Override extension to content type
16. Override encoding to content encoding
17. Handler path supports `r<w>x<h>/` prefixing instead of query params
18. Parse form file name into content encoding and content type
19. Add support for responding with content encoding depending on extension
20. Custom file copy method, Rocket's persist method does not work across devices
21. Custom NamedFile response (FileContent)
22. Refactor to use a route handler instead of a guard and handler
23. Virtual Object info endpoint
24. No longer rely upon public path on object
24. Create virtual object for content hash
25. Create virtual object for object if path is set
26. Upsert should update content encoding
27. Upsert should update content type
28. Use tokio async file for writing data and hashing files

## Next things to do

### Error Response
* Custom error type and error response (Soundness)

## Other
* Add HMAC key env for hashing content (Soundness)

### Meta Data
* Virtual Object tags (G2)
* Virtual Object path prefixes (G2)

### Content Types and Encoding
* Look for exact match in db before iterating over all
* Filter content type in database query (Performance)
* Determine content type and encoding by extension during upload (e.g. `.js.gz => text/javascript gzip`) (G1)
    > https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Encoding
    > https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Accept-Encoding

### Virtual Object Enhancements
* Virtual Object can list prioritized content type in case user content type is not specified (G1, G2)
* Better virtual object choice by resolution (G1, G2)
* Use transaction around inserts with last insert id (Soundness)

### Object enhancements
* Use transaction around persisting file (Soundness)
* Insert / Update custom headers (No Goal Alignment)
* Adjust diesel Object to use json map for headers

### Content Response
* Send headers on objects with custom named file response (No Goal Alignment)
* Store headers on objects (No Goal Alignment)
* Send custom headers on objects (No Goal Alignment)
* Last Modified (G1)
* Vary? (G1)
* CORS (G1)
* Content type no sniff (G1)
* age (G1)
* If Modified Since (G1)
* If None Match (G1)
* Support content range requests (G1)

### Dynamic Resize (G3)
* Find Exact resolution match, sort by lossless then lossy, filter by supported content types and content encodings
* Find Double (or higher) resolution match, sort by ...
* Find max resolution, sort by ...
* Load image and get dimensions (inspection)
  https://docs.rs/image/0.24.1/image/struct.ImageBuffer.html
  https://docs.rs/image/0.24.1/image/fn.load.html
* Add default bg color to vobj
* Add filter chain (json) to object table
* Add parent / derived object to object table
* Add parent path to upload function
* Add client provided filter chain to upload function
* Apply filter chain function (only resize supported)
* Save image to temp file with chosen content type
* Hash and create new Object (with relationship to vobj)
* refactor image loading, filters, and saving
* Async all of it from the request, have loading filters and saving eun on io thread
* Ingroduce semaphore so async processing does not consume too much memory
* Async identify dimensions of uploaded images and set width and height
* Add crop filter support
* Add blur filter support
* Consider blurhash (no rust encoder exists, c encoder looks relatively fine https://github.com/woltapp/blurhash/blob/master/C/encode.c )
* Add text overlay support

# Cryptography
* Plan signed urls (G4)

