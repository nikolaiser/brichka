//> using scala 2.13
//> using dep org.apache.spark::spark-sql::3.5.7

// brichka: exclude
import org.apache.spark.sql.functions._
import org.apache.spark.sql.{DataFrame, SparkSession}

val spark: SparkSession = ???
def display(df: DataFrame): Unit = ()
// brichka: include

//> using file fst.sc

// brichka: exclude
import fst._
import org.apache.spark.sql.functions._
// brichka: include
