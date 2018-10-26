#include <iostream>
#include <iomanip>
#include <vector>
#include <boost/random.hpp>
#include <boost/lexical_cast.hpp>
#include <alps/accumulators.hpp>

int main(int argc, char* argv[]){
  alps::accumulators::accumulator_set measurements;
  measurements << alps::accumulators::LogBinningAccumulator<double>("log");
  measurements << alps::accumulators::LogBinningAccumulator<double>("ran");
  boost::mt19937 eng(49135);
  boost::variate_generator<boost::mt19937&, boost::uniform_real<>> random_uniform(eng, boost::uniform_real<>());
  std::vector<double> log_data, ran_data;
  int N = 500;
  if(argc == 2){
    N = boost::lexical_cast<double>(argv[1]);
  }else{
    std::cerr << "usege: ./main number, N = 500 (Default)" << std::endl;
  }
  double x = 0;
  for(int i = 0; i < N; ++i){
    double next = x + random_uniform() - 0.5;
    if(abs(x) < abs(next)){
      double probability = exp((x*x - next*next) * 0.125);
      double dice = random_uniform();
      if(probability > dice){
	x = next;
      }
    }else{
      x = next;
    }
    measurements["log"] << x;
    double ran_cash = random_uniform();
    measurements["ran"] << ran_cash;

    log_data.push_back(x);
    ran_data.push_back(ran_cash);
  }
  alps::accumulators::result_set result(measurements);
  std::cout << "log " << std::setprecision(20) << result["log"] << std::endl;
  std::cout << "ran " <<  result["ran"] << std::endl;
  auto log = result["log"];
  auto ran = result["ran"];

  double vari_log = 0.0;
  double mean_log = 0.0;
  double vari_ran = 0.0;
  double mean_ran = 0.0;
  for(int i = 0; i < ran_data.size(); ++i){
    mean_log += log_data[i];
    mean_ran += ran_data[i];
  }
  mean_log /= log_data.size();
  mean_ran /= ran_data.size();
  for(int i = 0; i < ran_data.size(); i = i+pow(2,floor(log2(N))-7)){
    double bin_log = 0.0;
    double bin_ran = 0.0;
    for(int j = 0; j < pow(2,floor(log2(N))-7); ++j){
      bin_log += log_data[i+j];
      bin_ran += ran_data[i+j];
    }
    bin_log /= pow(2,floor(log2(N))-7);
    bin_ran /= pow(2,floor(log2(N))-7);

    vari_log += (bin_log - mean_log)*(bin_log - mean_log);
    vari_ran += (bin_ran - mean_ran)*(bin_ran - mean_ran);
  }
  double vari_naive = 0.0;
  for(int i = 0; i < ran_data.size(); ++i){
    vari_naive += (log_data[i] - mean_log)*(log_data[i]-mean_log);
  }
  vari_naive /= log_data.size()*(log_data.size()-1);
  
  vari_log /= log_data.size()/(pow(2,floor(log2(N))-7))*(log_data.size()/(pow(2,floor(log2(N))-7)) - 1);
  vari_ran /= log_data.size()/(pow(2,floor(log2(N))-7))*(log_data.size()/(pow(2,floor(log2(N))-7)) - 1);
  std::cout << "log error: " << sqrt(vari_log) << std::setprecision(20) << ", log tau: " << (vari_log/vari_naive - 1.0)*0.5 << std::endl;
  std::cout << "ran error: " << sqrt(vari_ran) << std::endl;
  std::cout << "naive_error: " << sqrt(vari_naive) << std::endl;
  return 0;
}
