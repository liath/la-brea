# La Brea

A `ReadSeek`able tar file builder (basically the inverse of the tar crate). The
interface to this should take a list of `ReadSeek`s and emit on reading, a well
formed tar file of their contents. Ideally we also do not store any of the file
content ourselves (of either the input or output) and instead read the input as
it is read by the caller through us.
