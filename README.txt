
                             _ __   __ _ ___| |_ ___  _ __
                            | '_ \ / _` / __| __/ _ \| '__|
                            | |_) | (_| \__ \ || (_) | |
                            | .__/ \__,_|___/\__\___/|_|
                            |_|

                       The pastebin that hopefully doesn't suck

Description
===========

 pastor was born out of frustration with other pastebins and their shortcomings

   - require an external database, hassel to set up.
   - have very unreliable mime type parsing leading to
     files being returned with the wrong mime type
   - won't show files in the browser but only prompt to download

 pastor tries to do better

   - ensure content-disposition headers are set to inline to display in browser
   - only interfere with mimetypes where needed
     - sets all text/* mime types to text/plain to avoid inline
       rendering of e.g. html and thus injection of external resources.
   - file extensions are for squishy humans
     - try to guess an extension based on the mime type
       but only if absolutely sure it's the correct one
     - file extensions can be set to whatever the user desires,
       they are ignored when accessing the paste.
   - easy to set up, single binary, no external dependencies
     - release binaries are staticly deployed, container image provided.
     - uses an embedded database, sled, to store paste metadata.
     - various templates like this page are compiled in by default
       but can be customized by providing external files.

Installation
============

 To run pastor with docker or podman:

   podman run -p 80:8000 -v <storage path>:/storage kube.cat/cocainefarm/pastor:latest

 To run pastor using the binary:

   curl -Lo pastor https://gitlab.com/api/v4/projects/17469937/packages/generic/pastor/0.8.0/pastor_amd64_static
   chmod +x pastor
   ROCKET_STORAGE_DIR=<storage path> ./pastor

Usage
=====

Files
-----
 Upload

  Upload files:
  -----------------------------------------------------------------------------------
  $ curl -F a=@vim.png b=@vim2.png https://{{ url }}/
  https://{{ url }}/picturedsalters aloofgremlins
  -----------------------------------------------------------------------------------
  The second string is a token needed to authenticate for
  modifying or deleteing your uploads.

 Retrieve

  Retrieve file:
  -----------------------------------------------------------------------------------
  GET https://{{ url }}/picturedsalters
  -----------------------------------------------------------------------------------

  View file with syntax highlighting:
  -----------------------------------------------------------------------------------
  GET https://{{ url }}/picturedsalters?lang=rust
  GET https://{{ url }}/picturedsalters?lang=auto (attempts detection)
  -----------------------------------------------------------------------------------

 Delete

  Delete file:
  -----------------------------------------------------------------------------------
  $ curl -X DELETE https://{{ url }}/picturedsalters?token=aloofgremlins
  or
  GET https://{{ url }}/delete/picturedsalters?token=aloofgremlins
  -----------------------------------------------------------------------------------

Expiry
======

 Files expire after an mount of time calculated based on the following formula:

    min_age + (-max_age + min_age) * (size / max_size - 1)^3

 Where min_age is 5 days, max_age is 365 days, and the maximum size is 512MiB.

  days
  350 |.
      | ..
      |  ..
  300 |-   ..
      |     ..
  250 |-      .
      |        ..
      |         ..
  200 |-          ..
      |             ..
      |               ..
  150 |-                ...
      |                   ...
      |                     ...
  100 |-                       ...
      |                           ....
   50 |-                            ......
      |                                   ......
      |                                         ............................
    0 +------------|-------------|------------|-------------|------------|-+
      0           100           200          300           400          500 MiB

GUI
=====

A graphical user interface is available at: https://{{ url }}/gui


Source
=====

The source code can be found at: https://gitlab.com/cocainefarm/pastor
