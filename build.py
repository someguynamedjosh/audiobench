#!/usr/bin/env python3

import argparse
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

parser = argparse.ArgumentParser(description='Does things.')
parser.add_argument(
    'job', help='Which job to execute. "jobs" will print available jobs.')
parser.add_argument('-c', '--clean', action='store_true',
                    help='Clean all intermediate files before starting.')
parser.add_argument('-r', '--release', action='store_true',
                    help='Use release profiles and optimizations wherever possible.')
parser.add_argument('-g', '--github-runner', action='store_true',
                    help='Activate extra steps necessary for building within the limitations of GitHub hosted runners.')

args = parser.parse_args()


def set_env(name, value):
    os.environ[name] = value


def get_env(name) -> str:
    if name not in os.environ.keys():
        return ''
    else:
        return os.environ[name]


def rmdir(path):
    shutil.rmtree(path, ignore_errors=True)


def mkdir(path):
    os.makedirs(path, exist_ok=True)


def cp(src, dst):
    shutil.copy2(src, dst)


def cpdir(src, dst):
    shutil.copytree(src, dst)


def should_skip_dep(name: str, version: int) -> bool:
    """Returns true if the dependency is already set up"""
    mkdir(PROJECT_ROOT.joinpath('dependencies', name))
    try:
        f = open(PROJECT_ROOT.joinpath(
            'dependencies', name, '__dependency__.txt'), 'r', encoding='utf8')
        current_version = int(f.read())
        f.close()
        should = current_version == version
        if should:
            print('Skipping dependency as it is already set up.')
        return should
    except:
        return False


def mark_dep_complete(name: str, version: int):
    """Makes it so that calling should_skip_dep in the future will return true."""
    f = open(PROJECT_ROOT.joinpath(
        'dependencies', name, '__dependency__.txt'), 'w', encoding='utf8')
    f.write(str(version))
    f.close()


def temp_clone(git_url: str, commit_id: str) -> tempfile.TemporaryDirectory:
    target = tempfile.TemporaryDirectory('', 'git_')
    print('Cloning ' + git_url)
    command(['git', 'clone', git_url, target.name])
    print('\nSwitching to commit ' + commit_id)
    command(['git', 'checkout', '-q', commit_id], target.name)
    return target


def command(args, working_dir=None):
    for index in range(0, len(args)):
        if type(args[index]) is not str:
            args[index] = str(args[index])
    real_working_dir = working_dir
    if real_working_dir is None:
        real_working_dir = PROJECT_ROOT
    proc = subprocess.Popen(args, cwd=real_working_dir)
    code = proc.wait()
    if code != 0:
        print('ERROR: The command "' + ' '.join(args) +
              '" failed with exit code ' + str(code) + '.')
        exit(code)


ON_WINDOWS = sys.platform in ['win32', 'cygwin']
ON_MAC = sys.platform.startswith('darwin')
ON_LINUX = sys.platform.startswith('linux')
DO_CLEAN = args.clean
DO_RELEASE = args.release
ON_GITHUB_RUNNER = args.github_runner
PROJECT_ROOT = Path(os.path.abspath(__file__)).parent
RUST_OUTPUT_DIR = PROJECT_ROOT.joinpath(
    'target', ['debug', 'release'][DO_RELEASE])
JUCE_FRONTEND_ROOT = PROJECT_ROOT.joinpath('components', 'juce_frontend')

cargo_toml = open('components/audiobench/Cargo.toml',
                    'r', encoding='utf8').read()
version_start = cargo_toml.find('version = "') + len('version = "')
version_end = cargo_toml.find('"', version_start)
CRATE_VERSION = cargo_toml[version_start:version_end]

# Tooling on windows expects forward slashes.
set_env('PROJECT_ROOT', str(PROJECT_ROOT).replace('\\', '/'))
set_env('RUST_OUTPUT_DIR', str(RUST_OUTPUT_DIR).replace('\\', '/'))
set_env('JULIA_DIR', str(PROJECT_ROOT.joinpath(
    'dependencies', 'julia')).replace('\\', '/'))
set_env('CRATE_VERSION', CRATE_VERSION)
if not ON_WINDOWS:
    set_env('LD_LIBRARY_PATH', get_env('LD_LIBRARY_PATH') + ':' +
            str(PROJECT_ROOT.joinpath('dependencies', 'julia', 'lib')))
mkdir(Path('dependencies'))


def print_jobs():
    print('Available jobs are as follows:')
    for job_name in JOBS:
        seperator = ': '
        if len(job_name) < 20:
            seperator += ' ' * (20 - len(job_name))
        print(job_name + seperator + JOBS[job_name].description)


def clean():
    command(['cargo', 'clean'])
    rmdir(PROJECT_ROOT.joinpath('artifacts'))
    rmdir(PROJECT_ROOT.joinpath('dependencies'))
    rmdir(JUCE_FRONTEND_ROOT.joinpath('_build'))


def build_clib():
    args = ['cargo', 'build', '-p', 'audiobench_clib']
    if DO_RELEASE:
        args.append('--release')
    command(args)


def remove_juce_splash():
    python = ['python3', 'python'][ON_WINDOWS]
    args = [python, JUCE_FRONTEND_ROOT.joinpath('remove_splash.py')]
    command(args, working_dir=JUCE_FRONTEND_ROOT)


def build_juce_frontend():
    mkdir(PROJECT_ROOT.joinpath('artifacts', 'bin'))
    mkdir(JUCE_FRONTEND_ROOT.joinpath('_build'))

    cmake_config = ['Debug', 'Release'][DO_RELEASE]
    if ON_WINDOWS:
        command(['cmake', '-GVisual Studio 16 2019', '-A', 'x64', '-Thost=x64',
                 '..'], working_dir=JUCE_FRONTEND_ROOT.joinpath('_build'))
        command(['cmake', '--build', '_build', '--config',
                 cmake_config], working_dir=JUCE_FRONTEND_ROOT)
    if ON_MAC or ON_LINUX:
        command(['cmake', '-Wno-dev', '-DCMAKE_BUILD_TYPE=' + ['Debug', 'Release']
                 [DO_RELEASE], '..'], working_dir=JUCE_FRONTEND_ROOT.joinpath('_build'))
        command(['cmake', '--build', '_build', '--config',
                 cmake_config], working_dir=JUCE_FRONTEND_ROOT)

    artifact_source = JUCE_FRONTEND_ROOT.joinpath('_build', 'Audiobench_artefacts', [
        'Debug', 'Release'][DO_RELEASE])
    standalone_source = artifact_source.joinpath('Standalone')
    vst3_source = artifact_source.joinpath(
        'VST3', 'Audiobench.vst3', 'Contents')
    au_source = None
    clib_source = RUST_OUTPUT_DIR.joinpath()

    artifact_target = PROJECT_ROOT.joinpath('artifacts', 'bin')
    standalone_target = artifact_target.joinpath()
    vst3_target = artifact_target.joinpath()
    au_source = None
    clib_target = artifact_target.joinpath()

    if ON_WINDOWS:
        standalone_source = standalone_source.joinpath('Audiobench.exe')
        vst3_source = vst3_source.joinpath('x86_64-win', 'Audiobench.vst3')
        clib_source = clib_source.joinpath('audiobench_clib.dll')

        standalone_target = standalone_target.joinpath(
            'Audiobench_Windows_x64_Standalone.exe')
        vst3_target = vst3_target.joinpath('Audiobench_Windows_x64_VST3.vst3')
        clib_target = clib_target.joinpath('audiobench_clib.dll')
    
    if ON_MAC:
        standalone_source = standalone_source.joinpath('Audiobench.app')
        vst3_source = artifact_source.joinpath('VST3', 'Audiobench.vst3')
        au_source = artifact_source.joinpath('AU', 'Audiobench.component')
        clib_source = clib_source.joinpath('libaudiobench_clib.dylib')

        standalone_target = standalone_target.joinpath(
            'Audiobench_MacOS_x64_Standalone.app')
        vst3_target = vst3_target.joinpath(
            'Audiobench_MacOS_x64_VST3.vst3')
        au_target = au_target.joinpath(
            'Audiobench_MacOS_x64_AU.component')
        clib_target = clib_target.joinpath('libaudiobench_clib.dylib')

    if ON_LINUX:
        standalone_source = standalone_source.joinpath('Audiobench')
        vst3_source = vst3_source.joinpath('x86_64-linux', 'Audiobench.so')
        clib_source = clib_source.joinpath('libaudiobench_clib.so')

        standalone_target = standalone_target.joinpath(
            'Audiobench_Linux_x64_Standalone.bin')
        vst3_target = vst3_target.joinpath('Audiobench_Linux_x64_VST3.vst3')
        clib_target = clib_target.joinpath('libaudiobench_clib.so')

    # Mac requires an extra packaging step whose output goes directly in artifacts/bin/. Other
    # platforms require copying the artifacts to the folder.
    if ON_MAC:
        # Add DS_Store and bg,png
        # NOTE: The DS_Store_VST3 file is just a copy of the Standalone file, never got around to
        # making an actual version of it.
        # bg_png_path = JUCE_FRONTEND_ROOT.joinpath('osx_stuff', 'bg.png')
        # for source in [standalone_source, vst3_source, au_source]:
        #     name = source.name
        #     ds_store_path = JUCE_FRONTEND_ROOT.joinpath('osx_stuff', 'DS_Store_' + name)
        #     mkdir(source.joinpath('.background'))
        #     cp(bg_png_path, source.joinpath('.background', 'bg.png'))
        #     cp(ds_store_path, source.joinpath('.DS_Store'))

        # Convert everything to zips.
        # command(['zip', '-r', artifact_target.joinpath(
        #     'Audiobench_MacOS_x64_Standalone.zip'), 'Audiobench.app'], working_dir=standalone_source)
        # command(['zip', '-r', artifact_target.joinpath(
        #     'Audiobench_MacOS_x64_VST3.zip'), 'Audiobench.vst3'], working_dir=vst3_source)
        # command(['zip', '-r', artifact_target.joinpath(
        #     'Audiobench_MacOS_x64_AU.zip'), 'Audiobench.component'], working_dir=au_source)
        cpdir(standalone_source, standalone_target)
        cpdir(vst3_source, vst3_target)
        cpdir(au_source, au_target)
    else:
        cp(standalone_source, standalone_target)
        cp(vst3_source, vst3_target)
    cp(clib_source, clib_target)


def build_installer():
    if ON_LINUX:
        command(['sh', 'build.sh'], PROJECT_ROOT.joinpath('components', 'installer_linux'))
    elif ON_MAC:
        command(['sh', 'build.sh'], PROJECT_ROOT.joinpath('components', 'installer_macos'))
    else:
        print('Not implemented alskdjlaksdj')
        # exit(1)


def run_standalone():
    artifact = 'Audiobench_'
    if ON_WINDOWS:
        artifact += 'Windows_x64_Standalone.exe'
    if ON_MAC:
        exit(1)
    if ON_LINUX:
        artifact += 'Linux_x64_Standalone.bin'
    command([PROJECT_ROOT.joinpath('artifacts', 'bin', artifact)])


def run_tests():
    args = ['cargo', 'test']
    if DO_RELEASE:
        args.append('--release')
    args.append('--')
    # Some of the tests test running Julia, which cannot be run multiple times on different threads.
    args.append('--test-threads=1')
    command(args)


def run_benchmark():
    args = ['cargo', 'run', '-p', 'benchmark']
    if DO_RELEASE:
        args.append('--release')
    command(args)


def check_version():
    import requests
    latest = requests.get(
        'https://joshua-maros.github.io/audiobench/latest.json').json()
    version = int(latest['version'])
    expected_version = version + 1
    good = True

    minor_version = int(CRATE_VERSION.split('.')[1].strip())
    if minor_version != expected_version:
        print('ERROR in components/audiobench/Cargo.toml:')
        print('Expected minor version to be ' +
              str(expected_version) + ' but found ' + str(minor_version))
        good = False

    latest_json = open('docs/website/src/latest.json',
                       'r', encoding='utf8').read()
    version_start = latest_json.find('"version": ') + len('"version": ')
    version_end = latest_json.find(',', version_start)
    latest_version = int(latest_json[version_start:version_end].strip())
    if latest_version != expected_version:
        print('ERROR in docs/website/src/latest.json:')
        print('Expected version to be ' + str(expected_version) +
              ' but found ' + str(latest_version))
        good = False

    if not good:
        exit(1)
    print('Version has been incremented correctly.')


def build_juce6_win():
    JUCE6_PREFIX = PROJECT_ROOT.joinpath('dependencies', 'juce6_built')
    slashed_prefix = str(JUCE6_PREFIX).replace('\\', '/')
    set_env('JUCE6_PREFIX', slashed_prefix)
    mkdir(JUCE6_PREFIX)
    working_dir = PROJECT_ROOT.joinpath('dependencies', 'juce')
    command(['cmake', '-Bcmake-build-install', '-DCMAKE_INSTALL_PREFIX={}'.format(
        slashed_prefix), '-GVisual Studio 16 2019', '-A', 'x64', '-Thost=x64'], working_dir=working_dir)
    command(['cmake', '--build', 'cmake-build-install',
             '--target', 'install'], working_dir=working_dir)
    set_env('JUCE_DIR', str(JUCE6_PREFIX.joinpath(
        'lib', 'cmake', 'JUCE-6.0.0')).replace('\\', '/'))


def pack_julia_package(git_url: str, commit_id: str, module_name: str):
    """Turns a Julia package (from a Git repository) into a single file which can more easily be
    embedded into an application.
    """
    repo_dir = temp_clone(git_url, commit_id)
    src_dir = Path(repo_dir.name).joinpath('src')
    packed_code = ''
    custom_include_name = '__packed_' + module_name + '_include__'
    custom_module_name = '__packed_' + module_name + '__'

    def stringify(text: str) -> str:
        return '"' + text.replace('\\', '\\\\').replace('"', '\\"').replace('\n', '\\n').replace('\r', '\\r').replace('\t', '\\t').replace('$', '\\$') + '"'

    packed_code += 'module ' + custom_module_name + '\n\n'
    packed_code += 'sources = Dict([\n'
    for src_file_path in src_dir.iterdir():
        f = open(src_file_path, 'r', encoding='utf8')
        code = f.read()
        f.close()
        code = code.replace(
            'include(', 'Main.' + custom_module_name + '.include(@__MODULE__, ')
        escaped_code = stringify(code)
        filename = stringify(str(src_file_path.relative_to(src_dir)))
        packed_code += '    (' + filename + ', ' + escaped_code + '),\n'
    packed_code += '])\n\n'
    packed_code += 'function include(mod, filename)\n'
    packed_code += '    code = sources[filename]\n'
    packed_code += '        include_string(mod, code, "packed/' + \
        module_name + '/" * filename)\n'
    packed_code += 'end\n\n'
    packed_code += 'end\n\n'
    packed_code += custom_module_name + \
        '.include(Main.UnpackedDependencies, "' + module_name + '.jl")\n'

    out_file = open(PROJECT_ROOT.joinpath(
        'dependencies', 'julia_packages', module_name + '.jl'), 'w', encoding='utf8')
    out_file.write(packed_code)
    out_file.close()


def get_julia_packages():
    if should_skip_dep('julia_packages', 1):
        return
    pack_julia_package('https://github.com/JuliaArrays/StaticArrays.jl',
                       'bfd1c051bbe6923261ee976a855dbc0676c02159', 'StaticArrays')
    print('Finished downloading all necessary Julia packages.')
    mark_dep_complete('julia_packages', 1)


def get_julia():
    if should_skip_dep('julia', 1):
        return
    target = tempfile.mktemp('.zip', 'julia')
    print('Downloading Julia 1.5.3...')
    if ON_WINDOWS:
        url = 'https://julialang-s3.julialang.org/bin/winnt/x64/1.5/julia-1.5.3-win64.zip'
        code = '''(new-object System.Net.WebClient).DownloadFile('$FILE','$DEST')'''.replace(
            '$FILE', url).replace('$DEST', target)
        command(['powershell', '-command', code])

        print('Extracting...')
        rmdir('dependencies/julia')
        mkdir('dependencies/')
        command(['tar', '-xf', target, '-C', 'dependencies/'])
        command(['powershell', '-command', 'Expand-Archive -Force \'' +
                 str(target) + '\' \'dependencies/\''])
        rmdir('dependencies/julia')
        command(['mv', 'dependencies/julia-1.5.3', 'dependencies/julia'])
        rmdir(target)
    if ON_MAC:
        url = 'https://julialang-s3.julialang.org/bin/mac/x64/1.5/julia-1.5.3-mac64.dmg'
        command(['curl', '-o', target, url])

        print('Extracting...')
        command(['hdiutil', 'attach', target])
        rmdir('dependencies/julia')
        command(
            ['cp', '-r', '/Volumes/Julia-1.5.3/Julia-1.5.app/Contents/Resources/julia/', 'dependencies/julia'])
        rmdir(target)
    if ON_LINUX:
        url = 'https://julialang-s3.julialang.org/bin/linux/x64/1.5/julia-1.5.3-linux-x86_64.tar.gz'
        command(['wget', url, '-O', target])

        print('Extracting...')
        command(['tar', '-xzf', target, '-C', 'dependencies/'])
        rmdir('dependencies/julia')
        command(['mv', 'dependencies/julia-1.5.3', 'dependencies/julia'])
        rmdir(target)
    mark_dep_complete('julia', 1)


def get_juce():
    if should_skip_dep('juce', 1):
        return
    # Version 6.0.1
    location = temp_clone('https://github.com/juce-framework/JUCE.git',
                          'a30f7357863a7d480a771e069abf56909cdf0e13')
    target = PROJECT_ROOT.joinpath('dependencies', 'juce')
    rmdir(target)
    if ON_WINDOWS:
        shutil.copytree(location.name, target)
    else:
        shutil.move(location.name, target)
    rmdir(target.joinpath('.git'))
    mark_dep_complete('juce', 1)


def get_dependencies():
    print('All dependencies set up successfully.')


def open_terminal():
    command([get_env('SHELL')])


class Job:
    def __init__(self, description, dependencies, executor):
        self.description = description
        self.dependencies = dependencies
        self.executor = executor


JOBS = {
    'jobs': Job('Print available jobs', [], print_jobs),
    'clean': Job('Delete all artifacts and intermediate files', [], clean),
    'dep_julia': Job('Build the "julia" dependency', [], get_julia),
    'dep_julia_packages': Job('Build the "julia_packages" dependency', [], get_julia_packages),
    'dep_juce': Job('Build the "juce" dependency', [], get_juce),
    'deps': Job('Download or build necessary dependencies', ['dep_julia', 'dep_julia_packages', 'dep_juce'], get_dependencies),
    'env': Job('Run a terminal after setting variables and installing deps', ['deps'], open_terminal),
    'remove_juce_splash': Job('Remove JUCE splash screen (Audiobench is GPLv3)', [], remove_juce_splash),
    'clib': Job('Build Audiobench as a dynamic library', ['deps'], build_clib),
    'juce_frontend': Job('Build the JUCE frontend for Audiobench', ['remove_juce_splash', 'clib'], build_juce_frontend),
    'installer': Job('Build a publishable installer', ['juce_frontend'], build_installer),
    'run': Job('Run the standalone version of Audiobench', ['juce_frontend'], run_standalone),
    'test': Job('Test all Rust components in the project', ['deps'], run_tests),
    'benchmark': Job('Run a benchmarking suite', ['deps'], run_benchmark),
    'check_version': Job('Ensures version numbers have been incremented', [], check_version),
}

if ON_WINDOWS:
    JOBS['juce6'] = Job('Build JUCE6 library (necessary on Windows)', [
        'remove_juce_splash'], build_juce6_win)
    JOBS['juce_frontend'].dependencies.append('juce6')

if args.job not in JOBS:
    print('ERROR: There is no job named "' + args.job + '"')
    print_jobs()
    exit(1)
job_order = [args.job]
job_index = 0
while job_index < len(job_order):
    for dependency in JOBS[job_order[job_index]].dependencies:
        job_order.append(dependency)
    job_index += 1
if DO_CLEAN:
    job_order.append('clean')
job_order.reverse()
clean_job_order = []
# Remove duplicates while preserving dependency relationships.
for job_id in job_order:
    if job_id not in clean_job_order:
        clean_job_order.append(job_id)
job_order = clean_job_order

print('The following steps will be taken:')
hr_index = 1
for job_id in job_order:
    print(str(hr_index) + '. ' + JOBS[job_id].description)
    hr_index += 1

hr_index = 1
for job_id in job_order:
    print('================================================================================')
    print('PERFORMING STEP ' + str(hr_index) +
          ': ' + JOBS[job_id].description)
    print('================================================================================')
    JOBS[job_id].executor()
    hr_index += 1

print('All steps completed successfully!')
exit(0)
