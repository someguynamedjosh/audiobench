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
    rmdir(dst)
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
set_env('CRATE_VERSION', CRATE_VERSION)
if not ON_WINDOWS:
    set_env('LD_LIBRARY_PATH', get_env('LD_LIBRARY_PATH') + ':' +
            str(PROJECT_ROOT.joinpath('artifacts', 'bin')))
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
    vst3_source = artifact_source.joinpath('VST3', 'Audiobench.vst3')
    au_source = None
    clib_source = RUST_OUTPUT_DIR.joinpath()

    artifact_target = PROJECT_ROOT.joinpath('artifacts', 'bin')
    standalone_target = artifact_target.joinpath()
    vst3_target = artifact_target.joinpath()
    au_target = artifact_target.joinpath()
    clib_target = artifact_target.joinpath()

    if ON_WINDOWS:
        standalone_source = standalone_source.joinpath('Audiobench.exe')
        clib_source = clib_source.joinpath('audiobench_clib.dll')

        standalone_target = standalone_target.joinpath(
            'Audiobench_Windows_x64_Standalone.exe')
        vst3_target = vst3_target.joinpath('Audiobench_Windows_x64_VST3.vst3')
        clib_target = clib_target.joinpath('audiobench_clib.dll')

    if ON_MAC:
        standalone_source = standalone_source.joinpath('Audiobench.app')
        au_source = artifact_source.joinpath('AU', 'Audiobench.component')
        clib_source = clib_source.joinpath('libaudiobench_clib.dylib')

        standalone_target = standalone_target.joinpath(
            'Audiobench_MacOS_x64_Standalone.app')
        vst3_target = vst3_target.joinpath(
            'Audiobench_MacOS_x64_VST3.vst3')
        au_target = au_target.joinpath(
            'Audiobench_MacOS_x64_AU.component')
        clib_target = clib_target.joinpath('libaudiobench_clib.dylib')

        # Change linkage paths.
        old_clib_path = '/usr/local/lib/libaudiobench_clib.0.1.0.dylib'
        new_clib_path = '/Library/Audiobench/libaudiobench_clib.dylib'
        files_to_change = [
            au_source.joinpath('Contents', 'MacOS', 'Audiobench'),
            standalone_source.joinpath('Contents', 'MacOS', 'Audiobench'),
            vst3_source.joinpath('Contents', 'MacOS', 'Audiobench'),
        ]
        for fpath in files_to_change:
            command(['install_name_tool', '-change',
                     old_clib_path, new_clib_path, fpath])

    if ON_LINUX:
        standalone_source = standalone_source.joinpath('Audiobench')
        clib_source = clib_source.joinpath('libaudiobench_clib.so')

        standalone_target = standalone_target.joinpath(
            'Audiobench_Linux_x64_Standalone.bin')
        vst3_target = vst3_target.joinpath('Audiobench_Linux_x64_VST3.vst3')
        clib_target = clib_target.joinpath('libaudiobench_clib.so.0')

    if ON_MAC:
        cpdir(standalone_source, standalone_target)
        cpdir(au_source, au_target)
    else:
        cp(standalone_source, standalone_target)
    cpdir(vst3_source, vst3_target)
    cp(clib_source, clib_target)


def build_installer():
    mkdir(PROJECT_ROOT.joinpath('artifacts', 'installer'))
    if ON_LINUX:
        command(['sh', 'build.sh'], PROJECT_ROOT.joinpath(
            'components', 'installer_linux'))
    if ON_MAC:
        command(['sh', 'build.sh'], PROJECT_ROOT.joinpath(
            'components', 'installer_macos'))
    if ON_WINDOWS:
        src_root = PROJECT_ROOT.joinpath('components', 'installer_windows')
        nsis_path = "C:/Program Files (x86)/NSIS/Bin/makensis.exe"
        command([nsis_path, 'main.nsi'], working_dir=src_root)
        cp(src_root.joinpath('AudiobenchInstaller.exe'),
           PROJECT_ROOT.joinpath('artifacts', 'installer'))


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
    last_version = [int(d) for d in latest['version'].split('.')]
    numeric_crate_version = [int(d) for d in CRATE_VERSION.split('.')]

    good = False
    for crate, last in zip(numeric_crate_version, last_version):
        if crate == last + 1:
            good = True
            break

    if good:
        print('Version has been incremented correctly.')
    else:
        print('ERROR in components/audiobench/Cargo.toml:')
        print('Version number was not incremented correctly.')
        print('Last version was ' +
              latest['version'] + ' but the crate version is ' + CRATE_VERSION)
        exit(1)


# This is only invoked in the CI script.
def set_release_version():
    # https://docs.github.com/en/actions/reference/workflow-commands-for-github-actions#using-workflow-commands-to-access-toolkit-functions
    print('::set-output name=RELEASE_NAME::' + CRATE_VERSION)


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
    'dep_juce': Job('Build the "juce" dependency', [], get_juce),
    'deps': Job('Download or build necessary dependencies', ['dep_juce'], get_dependencies),
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
if ON_GITHUB_RUNNER:
    JOBS['set_release_version'] = Job(
        'Set release version', [], set_release_version)

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
