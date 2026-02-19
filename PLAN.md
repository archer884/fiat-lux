# Plan

This documentation is intended for the use of AI coding agents in understanding
the intent and capabilities of this application.

`fiat-lux` ("the application") is an in-terminal Bible search program. It can
display verses in free (non-copyrighted) translations of the Bible; it can
perform a full-text search of the text of the Bible; and it can display those
verses which are determined to be most relevant to the user's queries.

The application is written in idiomatic Rust.

## Intended enhancement

Currently, we would like to supplement the program's output with a LINK to an
online Bible service for every verse that appears in the output, or for any
passage requested explicitly by the user.

The addition of this feature should have NO impact on existing features, and
the new feature should be added in such a way as to allow the inclusion of
additional online Bible providers in the future.
