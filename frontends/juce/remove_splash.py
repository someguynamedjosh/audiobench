#!/bin/env python

# JUCE allows removing the splash screen if your code is under GPLv3
path = 'juce_git/modules/juce_audio_processors/processors/juce_AudioProcessorEditor.cpp'
f = open(path, 'r')
content = f.read()
f.close()

start_label = '// BEGIN SECTION A'
end_label = '// END SECTION A'
start = content.find(start_label) + len(start_label)
end = content.find(end_label)
replace_with = '\n    // Audiobench is licensed under GPLv3'
replace_with += '\n    // splashScreen = new JUCESplashScreen (*this);'
replace_with += '\n    '
content = content[:start] + replace_with + content[end:]

f = open(path, 'w')
f.write(content)
f.close()
