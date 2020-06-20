#!/usr/bin/env python

# This script will repeated run the scp crate test suite and collect thread
# performance statistics, organized by network topology. It also saves a record
# of all output collected from the best and worst performance observed.

# suggested use: python ./tests/collect_statistics.py | tee -a statistics.dat

import sys, os
import signal
from subprocess import PIPE, Popen
from threading  import Thread
import time
from datetime import datetime

os.environ["SKIP_SLOW_TESTS"] = "1"
os.environ["MC_LOG"] = "debug" # debug will display slot advancement statistics
os.environ["RUST_BACKTRACE"] = "full"

command = 'cargo test --release --lib -- --test-threads=1 --nocapture'

try:
  from queue import Queue, Empty
except ImportError:
  from Queue import Queue, Empty  # support python 2.x

ON_POSIX = 'posix' in sys.builtin_module_names

def enqueue_output(out, queue):
  for line in iter(out.readline, b''):
    queue.put(line)
  out.close()

def save_output(filename, output):
  with open(filename,"w") as f:
    for line in output:
      f.write("{}".format(line))

if __name__ == '__main__':

  max_duration = 0
  min_duration = 1e100
  max_filename = ""
  min_filename = ""
  iteration = 0

  # abort iterations that have stalled
  max_single_iteration_time_to_wait_seconds = 1200
  max_dead_time_to_wait_seconds = 20

  while(True): # re-run "cargo test --release" until the user aborts the script

    p = Popen(command, stdout=PIPE, stderr=PIPE, bufsize=1, preexec_fn=os.setsid, close_fds=ON_POSIX, shell=True)

    q_stdout = Queue()
    t_stdout = Thread(target=enqueue_output, args=(p.stdout, q_stdout))
    t_stdout.daemon = True # thread dies with the program
    t_stdout.start()

    # per iteration tracking parameters
    output = []
    warn_count = 0
    erro_count = 0
    crit_count = 0

    iteration_start_time = time.time()
    last_read_time = time.time()

    while(True): # collect output for this iteration
      iteration_elapsed = time.time() - iteration_start_time
      dead_time_elapsed = time.time() - last_read_time

      dead_time_error = dead_time_elapsed > max_dead_time_to_wait_seconds
      test_time_error = iteration_elapsed > max_single_iteration_time_to_wait_seconds

      warn_count_error = warn_count > 10000
      erro_count_error = erro_count > 10000
      crit_count_error = crit_count > 10000

      count_error = warn_count_error or erro_count_error or crit_count_error

      if dead_time_error or count_error or test_time_error:
        # abort this iteration
        if not output:
          print("output is empty!")

        os.killpg(os.getpgid(p.pid), signal.SIGTERM)
        time.sleep(1.0)

        str = "unknown cause"
        str = "process output timed out" if dead_time_error else str
        str = "too many warnings" if count_error else str
        str = "test iteration timed out" if test_time_error else str

        formatted_line = "#ABORTED {0:d} cause: \"{2}\" \n".format(
          iteration,
          str,
        )
        output.append(formatted_line)
        sys.stdout.write(formatted_line)

        error_code = dead_time_error + count_error + test_time_error
        save_output("aborted_{0}_type_{1:d}.out".format(iteration, error_code), output)

        iteration_start_time = time.time()
        iteration += 1
        break


      # read one line of output from "cargo test --release" without blocking
      found_line = False
      line = ""
      try:
        line = q_stdout.get_nowait()
        found_line = True
      except Empty:
        pass

      if found_line:
        last_read_time = time.time()

        formatted_line = "{0:5.2f} :: {1}".format(iteration_elapsed, line)
        # sys.stdout.write(formatted_line)
        output.append(formatted_line) # collect all output lines

        # count unexpected log messages
        if line.find("WARN") > 0:
          warn_count += 1
          warn_line = "%%% WARN: {0}".format(line)
          sys.stdout.write(warn_line)
        if line.find("ERRO") > 0:
          erro_count += 1
          erro_line = "%%% ERRO: {0}".format(line)
          sys.stdout.write(erro_line)
        if line.find("CRIT") > 0:
          crit_count += 1
          crit_line = "%%% CRIT: {0}".format(line)
          sys.stdout.write(crit_line)

        # collect stats
        if line.find("(stats)") > 0:
          csv_values = line.split('(stats)')
          sys.stdout.write(csv_values[1])
          sys.stdout.flush()

        # check if this iteration is finished
        if line.find("build and test completed") > 0:

          duration = int(round((time.time() - iteration_start_time) * 1000))
          # update fastest and slowest runs
          if duration <= min_duration:
            min_duration = duration
            if os.path.exists(min_filename):
              os.remove(min_filename)
            min_filename = "min_{0}_{1}.out".format(iteration, duration)
            save_output(min_filename, output)

          if duration >= max_duration:
            max_duration = duration
            if os.path.exists(max_filename):
              os.remove(max_filename)
            max_filename = "max_{0}_{1}.out".format(iteration, duration)
            save_output(max_filename, output)

          # reset for next run
          output = []
          warn_count = 0
          erro_count = 0
          crit_count = 0
          error_type = 0

          time.sleep(0.100)
          sys.stdout.flush()

          iteration_start_time = time.time()
          iteration += 1
          break

