(ns titanic-ml.core
  (:require [clojure.java.io :as clj-io]
            [clojure.data.csv :as csv]
            [clojure.set :as set]
            [clojure.core.matrix.dataset :as ds]
            [clatern.decision-tree :as dt]
            [clojure.core.matrix :as m]
            [clojure.core.matrix.operators :as op]
            [incanter.core :as i]))


(def train-filename "../../datasets/titanic/train.csv")
(def test-filename "../../datasets/titanic/test.csv")

(defn load-csv [filename]
  (with-open [in-file (clj-io/reader filename)]
    (doall
      (csv/read-csv in-file))))

(defn csv-data->maps [csv-data]
  (map zipmap
       (->> (first csv-data)
            (map clojure.string/lower-case)
            (map keyword)
            repeat)
       (rest csv-data)))

(defn apply-transforms
  "Apply `transform-fns` to a single map `m`"
  [transform-fns m]
  (let [id-transforms (->> (keys m)
                           (map #(vector %1 identity))
                           (into {}))
        all-transforms (merge id-transforms transform-fns)]
    (merge-with #(%1 %2) all-transforms m)))

(def sex->int {"female" 0
               "male" 1})

(def embarkation->int {"S" 0
                       "C" 1
                       "Q" 2
                       ; TODO: Establish a better default value
                       ""  0})

(defn safe-read-string
  ([v]
    (safe-read-string v nil))
  ([v default]
   (cond
     (nil? v) default
     (empty? v) default
     :else (read-string v))))

(defn empty->int
  [v]
  (if (empty? v)
    0
    1))

(defn read-age
  [v]
   ; TODO: Establish a better default value for age
  (safe-read-string v 0))

(defn read-fare
  [v]
  ; TODO: Establish a better default value for fare
  (safe-read-string v 0)
  )

(def transform-fns
  {:passengerid   read-string
   :survived      read-string
   :pclass        read-string
   :sex           sex->int
   :age           read-age
   :sibsp         safe-read-string
   :parch         read-string
   :ticket        empty->int
   :fare          read-fare
   :cabin         empty->int
   :embarked      embarkation->int
   })

(def key-rename-map
  {:passengerid   :pax-id
   :pclass        :class
   :sibsp         :siblings-and-spouses
   :parch         :parents-and-children
   })

(defn file->dataset
  [filename]
  (->> filename
       load-csv
       csv-data->maps
       (map (partial apply-transforms transform-fns))
       (map #(set/rename-keys % key-rename-map))
       ds/dataset))


(defn add-family-size
  [dataset]
  (ds/add-column dataset :family-size
                 (op/+ (ds/column dataset :siblings-and-spouses)
                       (ds/column dataset :parents-and-children)
                       (repeat (count dataset) 1))))

(defn add-is-alone
  [dataset]
  (ds/add-column dataset :is-alone
                 (m/emap #(= % 1)
                         (ds/column dataset :family-size))))



(def train-dataset (ds/remove-columns (file->dataset train-filename)
                                      [:pax-id :name]))






(defn train-model
  [dataset]
  (let [X (ds/remove-columns dataset [:survived])
        y (ds/column dataset :survived)]
    (dt/cart X y))
  )

(def model (train-model train-dataset))


(def test-dataset (ds/remove-columns (file->dataset test-filename)
                                     [:pax-id :name]))


(def X (ds/remove-columns train-dataset [:survived]))

(def y (ds/column train-dataset :survived))