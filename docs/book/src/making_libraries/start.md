# Making Your Own Libraries

Libraries are a way to add extra modules and patches to Audiobench. They also
serve as a great way to share a large number of patches in a single file. To get
started making your own library, create a new folder in `Documents/Audiobench/`
and name it whatever you want. Create a file in that folder named
`library_info.yaml`. The contents of this file should follow this format:
```yaml
internal_name: (A name with no spaces or special characters)
pretty_name: (The name of your library that will be shown to the user)
description: (Some text to describe the contents of your library)
version: 0.1.0
```
The `version` field must follow [semantic versioning](https://semver.org/)
rules.

Once you have this file, opening the **Library Info** tab in Audiobench will
display your library alongside the factory and user libraries. You are now ready
to start adding items to your library!
