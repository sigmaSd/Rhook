**0.5.0**
- Hide compiling dynamic library stderr
- COUNTER doesnt need to be thread local

**0.4.0**
-  allow mutliple commands to set_hooks at the same time (including from different threads)
- Scaffold: `COUNTER` variable is now thread local

**0.3.0**
- Add all libc functions
- Rework API (see docs)

**0.2.2**
- Rework internals by using a macro to make the code cleaner
- Add GetEnv to libc hooks
- Add more examples
