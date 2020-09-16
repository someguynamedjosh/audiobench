#!/usr/bin/env python3

import argparse
import os
from pathlib import Path
import shutil
import subprocess
import sys

parser = argparse.ArgumentParser(description='Does things.')
parser.add_argument(
    'job', help='Which job to execute. "jobs" will print available jobs.')
parser.add_argument('-c', '--clean', action='store_true',
                    help='Clean all intermediate files before starting.')
parser.add_argument('-r', '--release', action='store_true',
                    help='Use release profiles and optimizations wherever possible.')

args = parser.parse_args()


def set_env(name, value):
    os.environ[name] = value


def get_env(name):
    return os.environ[name]


def rmdir(path):
    shutil.rmtree(path, ignore_errors=True)


def mkdir(path):
    os.makedirs(path, exist_ok=True)


def cp(src, dst):
    shutil.copy2(src, dst)


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
PROJECT_ROOT = Path(os.path.abspath(__file__)).parent
RUST_OUTPUT_DIR = PROJECT_ROOT.joinpath(
    'target', ['debug', 'release'][DO_RELEASE])
JUCE_FRONTEND_ROOT = PROJECT_ROOT.joinpath('components', 'juce_frontend')

# Tooling on windows expects forward slashes.
set_env('PROJECT_ROOT', str(PROJECT_ROOT).replace('\\', '/'))
set_env('RUST_OUTPUT_DIR', str(RUST_OUTPUT_DIR).replace('\\', '/'))


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
    rmdir(JUCE_FRONTEND_ROOT.joinpath('_build'))


def build_clib():
    args = ['cargo', 'build', '-p', 'audiobench-clib']
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
        command(['cmake', '-G"Visual Studio 16 2019"', '-A', 'x64', '-Thost=x64',
                 '..'], working_dir=JUCE_FRONTEND_ROOT.joinpath('_build'))
        command(['cmake', '--build', '_build', '--config',
                 cmake_config], working_dir=JUCE_FRONTEND_ROOT)
    if ON_MAC:
        exit(1)
    if ON_LINUX:
        command(['cmake', '-Wno-dev', '-DCMAKE_BUILD_TYPE=' + ['Debug', 'Release']
                 [DO_RELEASE], '..'], working_dir=JUCE_FRONTEND_ROOT.joinpath('_build'))
        command(['cmake', '--build', '_build', '--config',
                 cmake_config], working_dir=JUCE_FRONTEND_ROOT)

    artifact_source = JUCE_FRONTEND_ROOT.joinpath('_build', 'Audiobench_artefacts', [
        'Debug', 'Release'][DO_RELEASE])
    standalone_source = artifact_source.joinpath('Standalone')
    standalone_target = PROJECT_ROOT.joinpath('artifacts', 'bin')
    vst3_source = artifact_source.joinpath(
        'VST3', 'Audiobench.vst3', 'Contents')
    vst3_target = PROJECT_ROOT.joinpath('artifacts', 'bin')
    if ON_WINDOWS:
        standalone_source = standalone_source.joinpath('Audiobench.exe')
        standalone_target = standalone_target.joinpath(
            'Audiobench_Windows_x64_Standalone.exe')
        vst3_source = vst3_source.joinpath('x86_64-win', 'Audiobench.vst3')
        vst3_target = vst3_target.joinpath('Audiobench_Windows_x64_VST3.vst3')
    if ON_MAC:
        # Unimplemented
        exit(1)
    if ON_LINUX:
        standalone_source = standalone_source.joinpath('Audiobench')
        standalone_target = standalone_target.joinpath(
            'Audiobench_Linux_x64_Standalone.bin')
        vst3_source = vst3_source.joinpath('x86_64-linux', 'Audiobench.so')
        vst3_target = vst3_target.joinpath('Audiobench_Linux_x64_VST3.so')
    cp(standalone_source, standalone_target)
    cp(vst3_source, vst3_target)


def run_standalone():
    artifact = 'Audiobench_'
    if ON_WINDOWS:
        artifact += 'Windows_x64_Standalone.exe'
    if ON_MAC:
        exit(1)
    if ON_LINUX:
        artifact += 'Linux_x64_Standalone.bin'
    command([PROJECT_ROOT.joinpath('artifacts', 'bin', artifact)])


def build_juce6_win():
    JUCE6_PREFIX = JUCE_FRONTEND_ROOT.joinpath('juce6_built')
    slashed_prefix = str(JUCE6_PREFIX).replace('\\', '/')
    set_env('JUCE6_PREFIX', slashed_prefix)
    mkdir(JUCE6_PREFIX)
    command(['cmake', '-Bcmake-build-install', '-DCMAKE_INSTALL_PREFIX="{}"'.format(
        slashed_prefix), '-G"Visual Studio 16 2019"', '-A', 'x64', '-Thost=x64'], working_dir=JUCE_FRONTEND_ROOT.joinpath('juce_git'))
    command(['cmake', '--build', 'cmake-build-install', '--target',
             'install'], working_dir=JUCE_FRONTEND_ROOT.joinpath('juce_git'))
    set_env('JUCE_DIR', str(JUCE6_PREFIX.joinpath(
        'lib', 'cmake', 'JUCE-6.0.0')).replace('\\', '/'))


class Job:
    def __init__(self, description, dependencies, executor):
        self.description = description
        self.dependencies = dependencies
        self.executor = executor


JOBS = {
    'jobs': Job('Print available jobs', [], print_jobs),
    'clean': Job('Delete all artifacts and intermediate files', [], clean),
    'clib': Job('Build Audiobench as a static library', [], build_clib),
    'remove_juce_splash': Job('Remove JUCE splash screen (Audiobench is GPLv3)', [], remove_juce_splash),
    'juce_frontend': Job('Build the JUCE frontend for Audiobench', ['remove_juce_splash', 'clib'], build_juce_frontend),
    'run': Job('Run the standalone version of Audiobench', ['juce_frontend'], run_standalone),
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
