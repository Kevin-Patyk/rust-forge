struct Student {
    name: String,
    score: u32,
}

struct StudentGrade {
    name: String,
    score: u32,
    grade: Grade,
}

enum Grade {
    Excellent, // 90-100
    Good, // 80-89
    Average, // 70-79
    Poor, // 0-69
}

fn main() {
    let student_grades: Vec<Student> = vec![
        Student{name: "A".to_string(), score: 55},
        Student{name: "B".to_string(), score: 65},
        Student{name: "C".to_string(), score: 75},
        Student{name: "D".to_string(), score: 85},
        Student{name: "E".to_string(), score: 95},
    ];

    let mut student_grades_updated: Vec<StudentGrade> = Vec::new();

    for student in student_grades {
        match student.score {
            score if score >= 90 => {
                student_grades_updated.push(StudentGrade{name: student.name, score: student.score, grade: Grade::Excellent})
            }
            score if score >= 80 && score <= 89 => {
                student_grades_updated.push(StudentGrade{name: student.name, score: student.score, grade: Grade::Good})
            }
            score if score >= 70 && score <= 79 => {
                student_grades_updated.push(StudentGrade{name: student.name, score: student.score, grade: Grade::Average})
            }
            _ => {
                student_grades_updated.push(StudentGrade{name: student.name, score: student.score, grade: Grade::Poor})
            }
        }
    }

    student_grades_updated.sort_by(|a ,b| b.score.cmp(&a.score));

    for student in student_grades_updated {
        // Here we are taking variants of the enum and matching them
        // We are using match to convert each Grade enum variant into a printable string
        let grade_str = match student.grade {
            Grade::Excellent => "Excellent",
            Grade::Good => "Good",
            Grade::Average => "Average",
            Grade::Poor => "Poor",
        };

        println!("{}: {}, {}", student.name, student.score, grade_str)
    }

}
